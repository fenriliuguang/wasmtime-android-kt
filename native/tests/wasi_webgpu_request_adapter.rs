//! W2: wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter (transitional flat, true CM async).
//! Stub concurrent host returns a fixed non-zero u32; guest `run` must echo it.

use futures::channel::oneshot;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

fn register_request_adapter(linker: &mut Linker<()>) -> wasmtime::Result<()> {
    linker
        .instance("wasi:webgpu/webgpu@0.3.0-rc.2")?
        .func_wrap_concurrent("request-adapter", |_accessor, ()| {
            Box::pin(async move {
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                Ok((7u32,))
            })
        })?;
    Ok(())
}

#[test]
fn wasi_webgpu_request_adapter_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_request_adapter.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_request_adapter(&mut linker)?;

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
    assert_eq!(v, 7, "guest run must return stub adapter rep");
    Ok(())
}

#[test]
fn wasi_webgpu_request_adapter_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_request_adapter.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_request_adapter(&mut linker)?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 7,
        "guest run must return stub adapter rep via call_async"
    );
    Ok(())
}
