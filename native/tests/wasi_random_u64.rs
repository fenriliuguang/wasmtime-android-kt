//! WASI 0.3: wasi:random/random@0.3.0#get-random-u64 smoke.

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn wasi_random_get_u64_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/random_u64.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:random/random@0.3.0")?
        .func_wrap("get-random-u64", |_store, ()| {
            let mut bytes = [0u8; 8];
            getrandom::fill(&mut bytes).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
            Ok((u64::from_ne_bytes(bytes),))
        })?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u64,)>(&mut store, "run")?;
    let (a,) = func.call(&mut store, ())?;
    let (b,) = func.call(&mut store, ())?;
    // CSPRNG: vanishingly unlikely to collide; proves host is not a constant stub.
    assert_ne!(a, b, "two get-random-u64 calls returned the same value");
    Ok(())
}
