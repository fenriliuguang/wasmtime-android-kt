//! W1: wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter (transitional flat name).
//! Stub host returns a fixed non-zero u32; guest `run` must echo it.

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn wasi_webgpu_request_adapter_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_request_adapter.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:webgpu/webgpu@0.3.0-rc.2")?
        .func_wrap("request-adapter", |_store, ()| Ok((7u32,)))?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (rep,) = func.call(&mut store, ())?;
    assert_eq!(rep, 7, "guest run must return stub adapter rep");
    Ok(())
}
