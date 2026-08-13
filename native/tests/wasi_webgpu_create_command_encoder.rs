//! W3 slice: wasi:webgpu/webgpu@0.3.0-rc.2#device-create-command-encoder (transitional flat, sync).
//! Stub host: request-adapter → 7, adapter-request-device → 11, device-create-command-encoder → 17.

use futures::channel::oneshot;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

fn register_adapter_device_encoder(linker: &mut Linker<()>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.func_wrap_concurrent("request-adapter", |_accessor, ()| {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel::<()>();
            std::thread::spawn(move || {
                let _ = tx.send(());
            });
            let _ = rx.await;
            Ok((7u32,))
        })
    })?;
    webgpu.func_wrap_concurrent("adapter-request-device", |_accessor, (adapter,): (u32,)| {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel::<()>();
            std::thread::spawn(move || {
                let _ = tx.send(());
            });
            let _ = rx.await;
            assert_eq!(
                adapter, 7,
                "guest must pass adapter rep from request-adapter"
            );
            Ok((11u32,))
        })
    })?;
    webgpu.func_wrap(
        "device-create-command-encoder",
        |_caller, (device,): (u32,)| {
            assert_eq!(
                device, 11,
                "guest must pass device rep from adapter-request-device"
            );
            Ok((17u32,))
        },
    )?;
    Ok(())
}

#[test]
fn wasi_webgpu_create_command_encoder_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_create_command_encoder.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_adapter_device_encoder(&mut linker)?;

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
    assert_eq!(v, 17, "guest run must return stub encoder rep");
    Ok(())
}

#[test]
fn wasi_webgpu_create_command_encoder_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_create_command_encoder.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_adapter_device_encoder(&mut linker)?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 17, "guest run must return stub encoder rep via call_async");
    Ok(())
}
