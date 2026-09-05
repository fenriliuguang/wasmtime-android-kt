//! WASI 0.3: wasi:filesystem preopen + read/write smoke (Android sandbox subset).

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures::channel::oneshot;
use wasmtime::component::{
    Component, ComponentType, FutureReader, Lift, Linker, Lower, Resource, ResourceTable,
    ResourceType, Source, StreamConsumer, StreamReader, StreamResult,
};
use wasmtime::{Config, Engine, Store, StoreContextMut};

const PAYLOAD: &[u8] = b"P3FS";

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
    writer: Option<std::thread::JoinHandle<std::io::Result<()>>>,
}

struct TestHost {
    table: ResourceTable,
}

struct CollectConsumer {
    buf: Arc<Mutex<Vec<u8>>>,
    done: Option<oneshot::Sender<u32>>,
}

impl Drop for CollectConsumer {
    fn drop(&mut self) {
        if let Some(tx) = self.done.take() {
            let n = self.buf.lock().map(|b| b.len() as u32).unwrap_or(0);
            let _ = tx.send(n);
        }
    }
}

impl StreamConsumer<TestHost> for CollectConsumer {
    type Item = u8;

    fn poll_consume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        store: StoreContextMut<TestHost>,
        src: Source<'_, Self::Item>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let this = self.get_mut();
        let mut src = src.as_direct(store);
        let chunk = src.remaining();
        if chunk.is_empty() {
            if finish {
                return Poll::Ready(Ok(StreamResult::Cancelled));
            }
            let _ = cx;
            return Poll::Pending;
        }
        let n = chunk.len();
        this.buf.lock().unwrap().extend_from_slice(chunk);
        src.mark_read(n);
        Poll::Ready(Ok(StreamResult::Completed))
    }
}

fn sandbox_root() -> PathBuf {
    thread_local! {
        static DIR: PathBuf = {
            std::env::temp_dir()
                .join("wasmtime-android-kt-wasi-fs")
                .join(format!("{:?}", std::thread::current().id()))
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

fn fs_write_at(path: &std::path::Path, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
    let start = offset as usize;
    let mut existing = std::fs::read(path).unwrap_or_default();
    let end = start.saturating_add(bytes.len());
    if existing.len() < end {
        existing.resize(end, 0);
    }
    existing[start..end].copy_from_slice(bytes);
    std::fs::write(path, existing)
}

fn fs_read_from(path: &std::path::Path, offset: u64) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap_or_default();
    let start = (offset as usize).min(bytes.len());
    bytes[start..].to_vec()
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
        .push(FsDescriptor {
            path: child,
            writer: None,
        })
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
            "[method]descriptor.write-via-stream",
            |mut store, (desc, reader, offset): (Resource<FsDescriptor>, StreamReader<u8>, u64)| {
                let path = store.data_mut().table.get(&desc)?.path.clone();
                let (tx, rx) = oneshot::channel::<u32>();
                let buf = Arc::new(Mutex::new(Vec::new()));
                reader.pipe(
                    &mut store,
                    CollectConsumer {
                        buf: buf.clone(),
                        done: Some(tx),
                    },
                )?;
                let writer = std::thread::spawn(move || {
                    let _n = pollster::block_on(rx).unwrap_or(0);
                    let _ = _n;
                    let bytes = buf.lock().map(|b| b.clone()).unwrap_or_default();
                    if offset == 0 {
                        std::fs::write(&path, bytes)
                    } else {
                        fs_write_at(&path, offset, &bytes)
                    }
                });
                store.data_mut().table.get_mut(&desc)?.writer = Some(writer);
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(()))
                })?;
                Ok((fut,))
            },
        )?;
        types.func_wrap(
            "[method]descriptor.read-via-stream",
            |mut store, (desc, offset): (Resource<FsDescriptor>, u64)| {
                let entry = store.data_mut().table.get_mut(&desc)?;
                if let Some(h) = entry.writer.take() {
                    let _ = h.join();
                }
                let path = entry.path.clone();
                let bytes = fs_read_from(&path, offset);
                let reader = StreamReader::new(&mut store, bytes)?;
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(()))
                })?;
                Ok(((reader, fut),))
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
                writer: None,
            })?;
            Ok((vec![(resource, ".".to_string())],))
        })?;
    }
    Ok(())
}

#[test]
fn sandbox_join_rejects_escape() {
    assert!(matches!(sandbox_join(".."), Err(FsErrorCode::Access)));
    assert!(matches!(
        sandbox_join("../etc/passwd"),
        Err(FsErrorCode::Access)
    ));
    assert!(matches!(
        sandbox_join("/sdcard/x"),
        Err(FsErrorCode::Access)
    ));
    assert!(matches!(sandbox_join("a/../b"), Err(FsErrorCode::Access)));
    assert!(matches!(sandbox_join(""), Err(FsErrorCode::Invalid)));
    assert!(matches!(
        sandbox_join("x\0y"),
        Err(FsErrorCode::IllegalByteSequence)
    ));
    assert!(sandbox_join("p3fs.txt").is_ok());
}

#[test]
fn open_at_dotdot_returns_access() -> wasmtime::Result<()> {
    std::fs::create_dir_all(sandbox_root())?;
    let mut table = ResourceTable::new();
    let parent = table.push(FsDescriptor {
        path: sandbox_root(),
        writer: None,
    })?;
    let err = fs_open_child(&mut table, &parent, "..").unwrap_err();
    assert!(matches!(err, FsErrorCode::Access));
    Ok(())
}

#[test]
fn open_at_missing_parent_is_bad_descriptor() -> wasmtime::Result<()> {
    let mut table = ResourceTable::new();
    let dangling = Resource::<FsDescriptor>::new_own(0);
    let err = fs_open_child(&mut table, &dangling, "p3fs.txt").unwrap_err();
    assert!(matches!(err, FsErrorCode::BadDescriptor));
    Ok(())
}

#[test]
fn write_io_on_directory_is_not_unknown() {
    std::fs::create_dir_all(sandbox_root()).unwrap();
    let err = std::fs::write(sandbox_root(), b"x").unwrap_err();
    let code = fs_error_from_io(&err);
    assert!(
        matches!(
            code,
            FsErrorCode::IsDirectory | FsErrorCode::Io | FsErrorCode::Access
        ),
        "write-on-dir must map off unknown, got {code:?}"
    );
}

#[test]
fn wasi_filesystem_preopen_read_write_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/filesystem_preopen.wasm"
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
    assert_eq!(n, PAYLOAD.len() as u32);
    let on_disk = std::fs::read(sandbox_join("p3fs.txt").unwrap())?;
    assert_eq!(on_disk, PAYLOAD);
    Ok(())
}
