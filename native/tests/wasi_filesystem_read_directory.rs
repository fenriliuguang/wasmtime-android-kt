//! WASI 0.3: wasi:filesystem `read-directory` as a CM stream of directory entries.

use std::path::PathBuf;

use wasmtime::component::{
    Component, ComponentType, FutureReader, Lift, Linker, Lower, Resource, ResourceTable,
    ResourceType, StreamReader,
};
use wasmtime::{Config, Engine, Store};

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum FsErrorCode {
    #[component(name = "access")]
    Access,
    #[component(name = "already")]
    Already,
    #[component(name = "bad-descriptor")]
    BadDescriptor,
    #[component(name = "busy")]
    Busy,
    #[component(name = "deadlock")]
    Deadlock,
    #[component(name = "quota")]
    Quota,
    #[component(name = "exist")]
    Exist,
    #[component(name = "file-too-large")]
    FileTooLarge,
    #[component(name = "illegal-byte-sequence")]
    IllegalByteSequence,
    #[component(name = "in-progress")]
    InProgress,
    #[component(name = "interrupted")]
    Interrupted,
    #[component(name = "invalid")]
    Invalid,
    #[component(name = "io")]
    Io,
    #[component(name = "is-directory")]
    IsDirectory,
    #[component(name = "loop")]
    Loop,
    #[component(name = "too-many-links")]
    TooManyLinks,
    #[component(name = "message-size")]
    MessageSize,
    #[component(name = "name-too-long")]
    NameTooLong,
    #[component(name = "no-device")]
    NoDevice,
    #[component(name = "no-entry")]
    NoEntry,
    #[component(name = "no-lock")]
    NoLock,
    #[component(name = "insufficient-memory")]
    InsufficientMemory,
    #[component(name = "insufficient-space")]
    InsufficientSpace,
    #[component(name = "not-directory")]
    NotDirectory,
    #[component(name = "not-empty")]
    NotEmpty,
    #[component(name = "not-recoverable")]
    NotRecoverable,
    #[component(name = "unsupported")]
    Unsupported,
    #[component(name = "no-tty")]
    NoTty,
    #[component(name = "no-such-device")]
    NoSuchDevice,
    #[component(name = "overflow")]
    Overflow,
    #[component(name = "not-permitted")]
    NotPermitted,
    #[component(name = "pipe")]
    Pipe,
    #[component(name = "read-only")]
    ReadOnly,
    #[component(name = "invalid-seek")]
    InvalidSeek,
    #[component(name = "text-file-busy")]
    TextFileBusy,
    #[component(name = "cross-device")]
    CrossDevice,
    #[component(name = "other")]
    Other(Option<String>),
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum DescriptorType {
    #[component(name = "block-device")]
    BlockDevice,
    #[component(name = "character-device")]
    CharacterDevice,
    #[component(name = "directory")]
    Directory,
    #[component(name = "fifo")]
    Fifo,
    #[component(name = "symbolic-link")]
    SymbolicLink,
    #[component(name = "regular-file")]
    RegularFile,
    #[component(name = "socket")]
    Socket,
    #[component(name = "other")]
    Other(Option<String>),
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct DirectoryEntry {
    #[component(name = "type")]
    type_: DescriptorType,
    name: String,
}

fn fs_error_from_io(err: &std::io::Error) -> FsErrorCode {
    use std::io::ErrorKind::*;
    match err.kind() {
        NotFound => FsErrorCode::NoEntry,
        PermissionDenied => FsErrorCode::Access,
        AlreadyExists => FsErrorCode::Exist,
        InvalidInput => FsErrorCode::Invalid,
        Interrupted => FsErrorCode::Interrupted,
        OutOfMemory => FsErrorCode::InsufficientMemory,
        BrokenPipe => FsErrorCode::Pipe,
        Unsupported => FsErrorCode::Unsupported,
        IsADirectory => FsErrorCode::IsDirectory,
        NotADirectory => FsErrorCode::NotDirectory,
        DirectoryNotEmpty => FsErrorCode::NotEmpty,
        ReadOnlyFilesystem => FsErrorCode::ReadOnly,
        StorageFull => FsErrorCode::InsufficientSpace,
        FileTooLarge => FsErrorCode::FileTooLarge,
        QuotaExceeded => FsErrorCode::Quota,
        InvalidFilename => FsErrorCode::IllegalByteSequence,
        NotSeekable => FsErrorCode::InvalidSeek,
        _ => FsErrorCode::Io,
    }
}

struct FsDescriptor {
    path: PathBuf,
}

struct TestHost {
    table: ResourceTable,
}

fn sandbox_root() -> PathBuf {
    thread_local! {
        static DIR: PathBuf = {
            std::env::temp_dir()
                .join("wasmtime-android-kt-wasi-fs")
                .join(format!("dir-{:?}", std::thread::current().id()))
        };
    }
    DIR.with(|p| p.clone())
}

fn sandbox_join(rel: &str) -> Result<PathBuf, FsErrorCode> {
    if rel.is_empty() {
        return Err(FsErrorCode::Invalid);
    }
    if rel.contains('\0') {
        return Err(FsErrorCode::IllegalByteSequence);
    }
    let p = std::path::Path::new(rel);
    if p.components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(FsErrorCode::Access);
    }
    Ok(sandbox_root().join(p))
}

fn fs_read_dir_entries(path: &std::path::Path) -> Result<Vec<DirectoryEntry>, FsErrorCode> {
    let meta = std::fs::metadata(path).map_err(|e| fs_error_from_io(&e))?;
    if !meta.is_dir() {
        return Err(FsErrorCode::NotDirectory);
    }
    let mut out = Vec::new();
    for ent in std::fs::read_dir(path).map_err(|e| fs_error_from_io(&e))? {
        let ent = ent.map_err(|e| fs_error_from_io(&e))?;
        let os_name = ent.file_name();
        if os_name == "." || os_name == ".." {
            continue;
        }
        let name = os_name.to_string_lossy().into_owned();
        let ty = match ent.file_type() {
            Ok(ft) if ft.is_dir() => DescriptorType::Directory,
            Ok(ft) if ft.is_symlink() => DescriptorType::SymbolicLink,
            Ok(ft) if ft.is_file() => DescriptorType::RegularFile,
            _ => DescriptorType::Other(None),
        };
        out.push(DirectoryEntry { type_: ty, name });
    }
    Ok(out)
}

fn fs_open_child(
    table: &mut ResourceTable,
    parent: &Resource<FsDescriptor>,
    rel: &str,
) -> Result<Resource<FsDescriptor>, FsErrorCode> {
    let _ = table.get(parent).map_err(|_| FsErrorCode::BadDescriptor)?;
    let child = sandbox_join(rel)?;
    if !child.exists() {
        std::fs::write(&child, b"").map_err(|e| fs_error_from_io(&e))?;
    }
    table
        .push(FsDescriptor { path: child })
        .map_err(|_| FsErrorCode::InsufficientMemory)
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    {
        let mut types = linker.instance("wasi:filesystem/types@0.3.0")?;
        types.resource(
            "descriptor",
            ResourceType::host::<FsDescriptor>(),
            |mut store, rep| {
                let resource = Resource::<FsDescriptor>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        types.func_wrap(
            "[method]descriptor.open-at",
            |mut store, (desc, path): (Resource<FsDescriptor>, String)| match fs_open_child(
                &mut store.data_mut().table,
                &desc,
                &path,
            ) {
                Ok(child) => Ok((Ok(child),)),
                Err(code) => Ok((Err(code),)),
            },
        )?;
        types.func_wrap(
            "[method]descriptor.read-directory",
            |mut store, (desc,): (Resource<FsDescriptor>,)| {
                let listed = {
                    let table = &mut store.data_mut().table;
                    match table.get(&desc) {
                        Ok(entry) => fs_read_dir_entries(&entry.path.clone()),
                        Err(_) => Err(FsErrorCode::BadDescriptor),
                    }
                };
                match listed {
                    Ok(entries) => {
                        let reader = StreamReader::new(&mut store, entries)?;
                        let fut = FutureReader::new(&mut store, async move {
                            Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(()))
                        })?;
                        Ok(((reader, fut),))
                    }
                    Err(code) => {
                        let reader = StreamReader::new(&mut store, Vec::<DirectoryEntry>::new())?;
                        let fut = FutureReader::new(&mut store, async move {
                            Ok::<_, wasmtime::Error>(Err::<(), FsErrorCode>(code))
                        })?;
                        Ok(((reader, fut),))
                    }
                }
            },
        )?;
    }
    {
        let mut preopens = linker.instance("wasi:filesystem/preopens@0.3.0")?;
        preopens.resource(
            "descriptor",
            ResourceType::host::<FsDescriptor>(),
            |mut store, rep| {
                let resource = Resource::<FsDescriptor>::new_own(rep);
                store.data_mut().table.delete(resource)?;
                Ok(())
            },
        )?;
        preopens.func_wrap("get-directories", |mut store, ()| {
            std::fs::create_dir_all(sandbox_root())?;
            let resource = store.data_mut().table.push(FsDescriptor {
                path: sandbox_root(),
            })?;
            Ok((vec![(resource, ".".to_string())],))
        })?;
    }
    Ok(())
}

#[test]
fn read_dir_omits_dot_entries_and_lists_file() -> wasmtime::Result<()> {
    std::fs::create_dir_all(sandbox_root())?;
    std::fs::write(sandbox_join("hello.txt").unwrap(), b"")?;
    let entries = fs_read_dir_entries(&sandbox_root()).unwrap();
    assert!(entries.iter().any(|e| e.name == "hello.txt"));
    assert!(entries.iter().all(|e| e.name != "." && e.name != ".."));
    Ok(())
}

#[test]
fn wasi_filesystem_read_directory_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/filesystem_read_directory.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    register(&mut linker)?;

    let mut store = Store::new(
        &engine,
        TestHost {
            table: ResourceTable::new(),
        },
    );
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let n = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })?;
    assert_eq!(n, 1);
    Ok(())
}
