//! WASI 0.3: wasi:cli/command-shaped async `run` smoke (transitional u32 0=ok).
//! Guest imports existing `wasi:cli/stdout@0.3.0#write-via-stream` and writes `CMD\n`.

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures::channel::oneshot;
use wasmtime::component::{
    Component, ComponentType, FutureReader, Lift, Linker, Lower, Source, StreamConsumer,
    StreamReader, StreamResult,
};
use wasmtime::{Config, Engine, Store, StoreContextMut};

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum CliErrorCode {
    #[component(name = "unknown")]
    Unknown,
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

impl StreamConsumer<()> for CollectConsumer {
    type Item = u8;

    fn poll_consume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        store: StoreContextMut<()>,
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
            // Match cm.rs: Pending without self-wake (self-wake reenters on Android).
            let _ = cx;
            return Poll::Pending;
        }
        let n = chunk.len();
        this.buf.lock().unwrap().extend_from_slice(chunk);
        src.mark_read(n);
        Poll::Ready(Ok(StreamResult::Completed))
    }
}

fn register_stdout(linker: &mut Linker<()>) -> wasmtime::Result<()> {
    linker.instance("wasi:cli/stdout@0.3.0")?.func_wrap(
        "write-via-stream",
        |mut store: StoreContextMut<()>, (reader,): (StreamReader<u8>,)| {
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
                Ok::<_, wasmtime::Error>(Ok::<(), CliErrorCode>(()))
            })?;
            let _ = buf;
            Ok((fut,))
        },
    )?;
    Ok(())
}

fn load_component(engine: &Engine) -> wasmtime::Result<Component> {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/cli_command.wasm"
    ))?;
    Component::new(engine, bytes)
}

fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    Engine::new(&config)
}

#[test]
fn wasi_cli_command_run_concurrent() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_stdout(&mut linker)?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let v = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })?;
    assert_eq!(v, 0);
    Ok(())
}

#[test]
fn wasi_cli_command_call_async() -> wasmtime::Result<()> {
    let engine = engine()?;
    let component = load_component(&engine)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_stdout(&mut linker)?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 0);
    Ok(())
}
