//! WASI 0.3: wasi:http incoming-handler + P010 body stream smoke.

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures::channel::oneshot;
use wasmtime::component::{
    Component, ComponentType, FutureReader, Lift, Linker, Lower, Resource, ResourceTable,
    ResourceType, Source, StreamConsumer, StreamReader, StreamResult,
};
use wasmtime::{Config, Engine, Store, StoreContextMut};

const PAYLOAD: &[u8] = b"HBOD";

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum HttpErrorCode {
    #[component(name = "unknown")]
    Unknown,
}

struct HttpRequest {
    body: Vec<u8>,
}

struct HttpResponse {
    status: u16,
    body: Arc<Mutex<Vec<u8>>>,
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

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut types = linker.instance("wasi:http/types@0.3.0")?;
    types.resource(
        "request",
        ResourceType::host::<HttpRequest>(),
        |mut store, rep| {
            let resource = Resource::<HttpRequest>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    types.resource(
        "response",
        ResourceType::host::<HttpResponse>(),
        |mut store, rep| {
            let resource = Resource::<HttpResponse>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    types.func_wrap("[constructor]request", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpRequest {
            body: PAYLOAD.to_vec(),
        })?;
        Ok((resource,))
    })?;
    types.func_wrap("[constructor]response", |mut store, ()| {
        let resource = store.data_mut().table.push(HttpResponse {
            status: 200,
            body: Arc::new(Mutex::new(Vec::new())),
        })?;
        Ok((resource,))
    })?;
    types.func_wrap(
        "[method]response.status-code",
        |mut store, (resp,): (Resource<HttpResponse>,)| {
            Ok((store.data_mut().table.get(&resp)?.status,))
        },
    )?;
    types.func_wrap(
        "[static]request.consume-body",
        |mut store, (this,): (Resource<HttpRequest>,)| {
            let req = store.data_mut().table.delete(this)?;
            let reader = StreamReader::new(&mut store, req.body)?;
            let fut = FutureReader::new(&mut store, async move {
                Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
            })?;
            Ok(((reader, fut),))
        },
    )?;
    types.func_wrap(
        "[static]response.new",
        |mut store, (reader,): (StreamReader<u8>,)| {
            let buf = Arc::new(Mutex::new(Vec::new()));
            let (tx, rx) = oneshot::channel::<u32>();
            reader.pipe(
                &mut store,
                CollectConsumer {
                    buf: buf.clone(),
                    done: Some(tx),
                },
            )?;
            let resource = store.data_mut().table.push(HttpResponse {
                status: 200,
                body: buf,
            })?;
            let fut = FutureReader::new(&mut store, async move {
                let _n = rx.await.unwrap_or(0);
                Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
            })?;
            Ok(((resource, fut),))
        },
    )?;
    types.func_wrap(
        "[static]response.consume-body",
        |mut store, (this,): (Resource<HttpResponse>,)| {
            let resp = store.data_mut().table.delete(this)?;
            let bytes = resp.body.lock().map(|b| b.clone()).unwrap_or_default();
            let reader = StreamReader::new(&mut store, bytes)?;
            let fut = FutureReader::new(&mut store, async move {
                Ok::<_, wasmtime::Error>(Ok::<(), HttpErrorCode>(()))
            })?;
            Ok(((reader, fut),))
        },
    )?;
    Ok(())
}

fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    Engine::new(&config)
}

fn load_component(engine: &Engine, file: &str) -> wasmtime::Result<Component> {
    let path = format!("{}/../fixtures/wasi/{file}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(path)?;
    Component::new(engine, bytes)
}

fn new_store(engine: &Engine) -> Store<TestHost> {
    Store::new(
        engine,
        TestHost {
            table: ResourceTable::new(),
        },
    )
}

fn call_run(engine: &Engine, file: &str) -> wasmtime::Result<u32> {
    let component = load_component(engine, file)?;
    let mut linker = Linker::new(engine);
    register(&mut linker)?;
    let mut store = new_store(engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })
}

#[test]
fn wasi_http_handler_run_returns_200() -> wasmtime::Result<()> {
    let engine = engine()?;
    assert_eq!(call_run(&engine, "http_handler.wasm")?, 200);
    Ok(())
}

#[test]
fn wasi_http_incoming_handler_export() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine, "http_handler.wasm")?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let status = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u16> {
                let req = accessor.with(|mut access| {
                    access.data_mut().table.push(HttpRequest {
                        body: PAYLOAD.to_vec(),
                    })
                })?;
                let idx = accessor.with(|mut access| {
                    let inst = instance
                        .get_export_index(&mut access, None, "wasi:http/incoming-handler@0.3.0")
                        .ok_or_else(|| {
                            wasmtime::Error::msg("missing wasi:http/incoming-handler@0.3.0")
                        })?;
                    instance
                        .get_export_index(&mut access, Some(&inst), "handle")
                        .ok_or_else(|| wasmtime::Error::msg("missing handle"))
                })?;
                let func = accessor.with(|mut access| {
                    instance.get_typed_func::<
                        (Resource<HttpRequest>,),
                        (Result<Resource<HttpResponse>, HttpErrorCode>,),
                    >(&mut access, idx)
                })?;
                let (result,) = func.call_concurrent(accessor, (req,)).await?;
                let resp = result.map_err(|_| wasmtime::Error::msg("handle err"))?;
                accessor.with(|mut access| Ok(access.data_mut().table.get(&resp)?.status))
            })
            .await?
    })?;
    assert_eq!(status, 200);
    Ok(())
}

#[test]
fn wasi_http_body_stream_echo() -> wasmtime::Result<()> {
    let engine = engine()?;
    assert_eq!(call_run(&engine, "http_body.wasm")?, PAYLOAD.len() as u32);
    Ok(())
}
