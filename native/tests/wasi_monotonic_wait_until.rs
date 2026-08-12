//! WASI 0.3: wasi:clocks/monotonic-clock@0.3.0#wait-until smoke (CM async).

use std::sync::OnceLock;
use std::time::Instant;

use futures::channel::oneshot;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

fn register_wait_until(linker: &mut Linker<()>) -> wasmtime::Result<()> {
    static MONOTONIC_START: OnceLock<Instant> = OnceLock::new();
    let mut clocks = linker.instance("wasi:clocks/monotonic-clock@0.3.0")?;
    clocks.func_wrap("now", |_store, ()| {
        let start = MONOTONIC_START.get_or_init(Instant::now);
        Ok((start.elapsed().as_nanos() as u64,))
    })?;
    clocks.func_wrap_concurrent("wait-until", |_accessor, (when,): (u64,)| {
        Box::pin(async move {
            let start = MONOTONIC_START.get_or_init(Instant::now);
            let now = start.elapsed().as_nanos() as u64;
            let sleep_ns = when.saturating_sub(now).min(1_000_000_000);
            let (tx, rx) = oneshot::channel::<()>();
            std::thread::spawn(move || {
                if sleep_ns > 0 {
                    std::thread::sleep(std::time::Duration::from_nanos(sleep_ns));
                }
                let _ = tx.send(());
            });
            let _ = rx.await;
            Ok(())
        })
    })?;
    Ok(())
}

#[test]
fn wasi_monotonic_clock_wait_until_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/monotonic_wait_until.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_wait_until(&mut linker)?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let v = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor.with(|mut access| {
                    instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                })?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })?;
    assert_eq!(v, 1);
    Ok(())
}

#[test]
fn wasi_monotonic_clock_wait_until_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/monotonic_wait_until.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    register_wait_until(&mut linker)?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1);
    Ok(())
}
