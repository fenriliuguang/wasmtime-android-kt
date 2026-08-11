use futures::channel::oneshot;
use wasmtime::component::{Component, FutureReader, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn m2_async_get_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/m2/async_get.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<()> = Linker::new(&engine);
    linker.root().func_wrap_concurrent("get", |accessor, ()| {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel::<u32>();
            let mut reader = accessor.with(|mut access| {
                FutureReader::new(&mut access, async move {
                    match rx.await {
                        Ok(v) => Ok(Some(v)),
                        Err(_) => Err(wasmtime::Error::msg("rejected")),
                    }
                })
            })?;
            tx.send(42).unwrap();
            accessor.with(|mut access| reader.close(&mut access))?;
            Ok((42u32,))
        })
    })?;
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
    assert_eq!(v, 42);
    Ok(())
}

#[test]
fn m2_async_get_simple_concurrent() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/m2/async_get.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<()> = Linker::new(&engine);
    linker.root().func_wrap_concurrent("get", |_accessor, ()| {
        Box::pin(async move { Ok((42u32,)) })
    })?;
    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;

    // Path A: call_async
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (a,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(a, 42);

    Ok(())
}
