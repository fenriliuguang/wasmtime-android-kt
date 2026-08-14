//! W3 slice: wasi:webgpu/webgpu@0.3.0-rc.2#render-pass-end (transitional flat, sync).
//! Stub host: request-adapter → 7, adapter-request-device → 11,
//! device-create-command-encoder → 17, begin-render-pass-clear(17, stub view 23) → 29;
//! render-pass-end asserts pass 29.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures::channel::oneshot;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

fn register_adapter_device_render_pass_end(
    linker: &mut Linker<()>,
    ended: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
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
    webgpu.func_wrap(
        "command-encoder-begin-render-pass-clear",
        |_caller, (encoder, view): (u32, u32)| {
            assert_eq!(
                encoder, 17,
                "guest must pass encoder rep from device-create-command-encoder"
            );
            assert_eq!(view, 23, "guest must pass transitional stub view 23");
            Ok((29u32,))
        },
    )?;
    webgpu.func_wrap("render-pass-end", move |_caller, (pass,): (u32,)| {
        assert_eq!(
            pass, 29,
            "guest must pass render-pass rep from begin-render-pass-clear"
        );
        ended.store(true, Ordering::SeqCst);
        Ok(())
    })?;
    Ok(())
}

#[test]
fn wasi_webgpu_render_pass_end_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_render_pass_end.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let ended = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<()> = Linker::new(&engine);
    register_adapter_device_render_pass_end(&mut linker, ended.clone())?;

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
    assert_eq!(v, 29, "guest run must return stub render-pass rep");
    assert!(
        ended.load(Ordering::SeqCst),
        "guest must call render-pass-end"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_render_pass_end_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_render_pass_end.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let ended = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<()> = Linker::new(&engine);
    register_adapter_device_render_pass_end(&mut linker, ended.clone())?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 29,
        "guest run must return stub render-pass rep via call_async"
    );
    assert!(
        ended.load(Ordering::SeqCst),
        "guest must call render-pass-end via call_async"
    );
    Ok(())
}
