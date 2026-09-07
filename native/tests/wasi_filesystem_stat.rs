//! WASI 0.3: wasi:filesystem `stat` / `stat-at` on the sandbox descriptor.

use std::path::PathBuf;

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
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

flags! {
    PathFlags {
        #[component(name = "symlink-follow")]
        const SYMLINK_FOLLOW;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct Instant {
    seconds: i64,
    nanoseconds: u32,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct DescriptorStat {
    #[component(name = "type")]
    type_: DescriptorType,
    #[component(name = "link-count")]
    link_count: u64,
    size: u64,
    #[component(name = "data-access-timestamp")]
    data_access_timestamp: Option<Instant>,
    #[component(name = "data-modification-timestamp")]
    data_modification_timestamp: Option<Instant>,
    #[component(name = "status-change-timestamp")]
    status_change_timestamp: Option<Instant>,
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
                .join(format!("stat-{:?}", std::thread::current().id()))
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

fn system_time_to_fs_instant(t: std::time::SystemTime) -> Instant {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => Instant {
            seconds: d.as_secs() as i64,
            nanoseconds: d.subsec_nanos(),
        },
        Err(e) => {
            let d = e.duration();
            Instant {
                seconds: -(d.as_secs() as i64),
                nanoseconds: d.subsec_nanos(),
            }
        }
    }
}

fn fs_descriptor_type(meta: &std::fs::Metadata) -> DescriptorType {
    let ft = meta.file_type();
    if ft.is_dir() {
        DescriptorType::Directory
    } else if ft.is_symlink() {
        DescriptorType::SymbolicLink
    } else if ft.is_file() {
        DescriptorType::RegularFile
    } else {
        DescriptorType::Other(None)
    }
}

fn fs_link_count(meta: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        meta.nlink()
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        1
    }
}

fn fs_ctime(meta: &std::fs::Metadata) -> Option<Instant> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let nsec = meta.ctime_nsec();
        if nsec < 0 {
            return None;
        }
        Some(Instant {
            seconds: meta.ctime(),
            nanoseconds: nsec as u32,
        })
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        None
    }
}

fn fs_stat_path(path: &std::path::Path, follow: bool) -> Result<DescriptorStat, FsErrorCode> {
    let meta = if follow {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    }
    .map_err(|e| fs_error_from_io(&e))?;
    Ok(DescriptorStat {
        type_: fs_descriptor_type(&meta),
        link_count: fs_link_count(&meta),
        size: meta.len(),
        data_access_timestamp: meta.accessed().ok().map(system_time_to_fs_instant),
        data_modification_timestamp: meta.modified().ok().map(system_time_to_fs_instant),
        status_change_timestamp: fs_ctime(&meta),
    })
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
            "[method]descriptor.stat",
            |mut store, (desc,): (Resource<FsDescriptor>,)| {
                let table = &mut store.data_mut().table;
                let entry = match table.get(&desc) {
                    Ok(e) => e,
                    Err(_) => return Ok((Err(FsErrorCode::BadDescriptor),)),
                };
                let path = entry.path.clone();
                match fs_stat_path(&path, true) {
                    Ok(st) => Ok((Ok(st),)),
                    Err(code) => Ok((Err(code),)),
                }
            },
        )?;
        types.func_wrap(
            "[method]descriptor.stat-at",
            |mut store, (desc, flags, path): (Resource<FsDescriptor>, PathFlags, String)| {
                let table = &mut store.data_mut().table;
                if table.get(&desc).is_err() {
                    return Ok((Err(FsErrorCode::BadDescriptor),));
                }
                let joined = match sandbox_join(&path) {
                    Ok(p) => p,
                    Err(code) => return Ok((Err(code),)),
                };
                match fs_stat_path(&joined, flags.contains(PathFlags::SYMLINK_FOLLOW)) {
                    Ok(st) => Ok((Ok(st),)),
                    Err(code) => Ok((Err(code),)),
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
fn stat_path_maps_dir_and_file() -> wasmtime::Result<()> {
    std::fs::create_dir_all(sandbox_root())?;
    let file = sandbox_join("hello.txt").unwrap();
    std::fs::write(&file, b"")?;
    let dir = fs_stat_path(&sandbox_root(), true).unwrap();
    assert!(matches!(dir.type_, DescriptorType::Directory));
    let st = fs_stat_path(&file, true).unwrap();
    assert!(matches!(st.type_, DescriptorType::RegularFile));
    assert_eq!(st.size, 0);
    Ok(())
}

#[test]
fn stat_at_dotdot_returns_access() -> wasmtime::Result<()> {
    std::fs::create_dir_all(sandbox_root())?;
    let mut table = ResourceTable::new();
    let parent = table.push(FsDescriptor {
        path: sandbox_root(),
    })?;
    let _ = table.get(&parent)?;
    let err = sandbox_join("..").unwrap_err();
    assert!(matches!(err, FsErrorCode::Access));
    Ok(())
}

#[test]
fn wasi_filesystem_stat_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/filesystem_stat.wasm"
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
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (n,) = func.call(&mut store, ())?;
    assert_eq!(n, 1);
    Ok(())
}
