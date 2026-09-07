//! WASI 0.3: wasi:filesystem `set-times` / `set-times-at` on sandbox files.

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

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct Instant {
    seconds: i64,
    nanoseconds: u32,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
enum NewTimestamp {
    #[component(name = "no-change")]
    NoChange,
    #[component(name = "now")]
    Now,
    #[component(name = "timestamp")]
    Timestamp(Instant),
}

flags! {
    PathFlags {
        #[component(name = "symlink-follow")]
        const SYMLINK_FOLLOW;
    }
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
                .join(format!("times-{:?}", std::thread::current().id()))
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

fn instant_to_system_time(i: Instant) -> Result<std::time::SystemTime, FsErrorCode> {
    use std::time::{Duration, UNIX_EPOCH};
    if i.nanoseconds >= 1_000_000_000 {
        return Err(FsErrorCode::Invalid);
    }
    if i.seconds >= 0 {
        UNIX_EPOCH
            .checked_add(Duration::new(i.seconds as u64, i.nanoseconds))
            .ok_or(FsErrorCode::Overflow)
    } else {
        UNIX_EPOCH
            .checked_sub(Duration::new((-i.seconds) as u64, i.nanoseconds))
            .ok_or(FsErrorCode::Overflow)
    }
}

fn fs_apply_new_timestamp(
    times: std::fs::FileTimes,
    ts: NewTimestamp,
    accessed: bool,
) -> Result<(std::fs::FileTimes, bool), FsErrorCode> {
    match ts {
        NewTimestamp::NoChange => Ok((times, false)),
        NewTimestamp::Now => {
            let t = std::time::SystemTime::now();
            Ok((
                if accessed {
                    times.set_accessed(t)
                } else {
                    times.set_modified(t)
                },
                true,
            ))
        }
        NewTimestamp::Timestamp(i) => {
            let t = instant_to_system_time(i)?;
            Ok((
                if accessed {
                    times.set_accessed(t)
                } else {
                    times.set_modified(t)
                },
                true,
            ))
        }
    }
}

fn fs_set_times(
    path: &std::path::Path,
    access: NewTimestamp,
    modify: NewTimestamp,
) -> Result<(), FsErrorCode> {
    let file = std::fs::File::open(path).map_err(|e| fs_error_from_io(&e))?;
    let (times, a) = fs_apply_new_timestamp(std::fs::FileTimes::new(), access, true)?;
    let (times, m) = fs_apply_new_timestamp(times, modify, false)?;
    if a || m {
        file.set_times(times).map_err(|e| fs_error_from_io(&e))?;
    }
    Ok(())
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
            "[method]descriptor.set-times",
            |mut store, (desc, access, modify): (Resource<FsDescriptor>, NewTimestamp, NewTimestamp)| {
                let path = match store.data_mut().table.get(&desc) {
                    Ok(entry) => entry.path.clone(),
                    Err(_) => return Ok((Err(FsErrorCode::BadDescriptor),)),
                };
                match fs_set_times(&path, access, modify) {
                    Ok(()) => Ok((Ok(()),)),
                    Err(code) => Ok((Err(code),)),
                }
            },
        )?;
        types.func_wrap(
            "[method]descriptor.set-times-at",
            |mut store,
             (desc, flags, path, access, modify): (
                Resource<FsDescriptor>,
                PathFlags,
                String,
                NewTimestamp,
                NewTimestamp,
            )| {
                let _ = flags;
                if store.data_mut().table.get(&desc).is_err() {
                    return Ok((Err(FsErrorCode::BadDescriptor),));
                }
                let joined = match sandbox_join(&path) {
                    Ok(p) => p,
                    Err(code) => return Ok((Err(code),)),
                };
                match fs_set_times(&joined, access, modify) {
                    Ok(()) => Ok((Ok(()),)),
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
fn wasi_filesystem_set_times_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/filesystem_set_times.wasm"
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
