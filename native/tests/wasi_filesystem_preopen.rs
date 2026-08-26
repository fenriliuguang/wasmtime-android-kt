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

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum FsErrorCode {
    #[component(name = "unknown")]
    Unknown,
    #[component(name = "access")]
    Access,
}

struct FsDescriptor {
    path: PathBuf,
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
    std::env::temp_dir().join("wasmtime-android-kt-wasi-fs")
}

fn sandbox_join(rel: &str) -> Result<PathBuf, FsErrorCode> {
    if rel.is_empty() || rel.contains('\0') {
        return Err(FsErrorCode::Access);
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
                let fut = FutureReader::new(&mut store, async move {
                    let _n = match rx.await {
                        Ok(n) => n,
                        Err(_) => 0,
                    };
                    let bytes = buf.lock().map(|b| b.clone()).unwrap_or_default();
                    let wrote = if offset == 0 {
                        std::fs::write(&path, bytes)
                    } else {
                        fs_write_at(&path, offset, &bytes)
                    };
                    match wrote {
                        Ok(()) => Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(())),
                        Err(_) => Ok(Err(FsErrorCode::Unknown)),
                    }
                })?;
                Ok((fut,))
            },
        )?;
        types.func_wrap(
            "[method]descriptor.read-via-stream",
            |mut store, (desc, offset): (Resource<FsDescriptor>, u64)| {
                let path = store.data_mut().table.get(&desc)?.path.clone();
                let bytes = fs_read_from(&path, offset);
                let reader = StreamReader::new(&mut store, bytes)?;
                let fut = FutureReader::new(&mut store, async move {
                    Ok::<_, wasmtime::Error>(Ok::<(), FsErrorCode>(()))
                })?;
                Ok(((reader, fut),))
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
            let path =
                sandbox_join("p3fs.txt").map_err(|_| wasmtime::Error::msg("sandbox join"))?;
            if !path.exists() {
                std::fs::write(&path, b"")?;
            }
            let resource = store.data_mut().table.push(FsDescriptor { path })?;
            Ok((vec![(resource, "p3fs.txt".to_string())],))
        })?;
    }
    Ok(())
}

#[test]
fn sandbox_join_rejects_escape() {
    assert!(sandbox_join("..").is_err());
    assert!(sandbox_join("../etc/passwd").is_err());
    assert!(sandbox_join("/sdcard/x").is_err());
    assert!(sandbox_join("a/../b").is_err());
    assert!(sandbox_join("p3fs.txt").is_ok());
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
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (n,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(n, PAYLOAD.len() as u32);
    let on_disk = std::fs::read(sandbox_join("p3fs.txt").unwrap())?;
    assert_eq!(on_disk, PAYLOAD);
    Ok(())
}
