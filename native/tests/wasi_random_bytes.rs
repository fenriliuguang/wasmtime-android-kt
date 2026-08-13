//! WASI 0.3: wasi:random/random@0.3.0#get-random-bytes smoke.

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn wasi_random_get_bytes_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/random_bytes.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:random/random@0.3.0")?
        .func_wrap("get-random-bytes", |_store, (len,): (u64,)| {
            let n = (len as usize).min(4096);
            let mut bytes = vec![0u8; n];
            if n > 0 {
                getrandom::fill(&mut bytes).map_err(|e| wasmtime::Error::msg(e.to_string()))?;
            }
            Ok((bytes,))
        })?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u64,)>(&mut store, "run")?;
    let (a,) = func.call(&mut store, ())?;
    let (b,) = func.call(&mut store, ())?;
    // CSPRNG: vanishingly unlikely to collide; guest packs 8 bytes LE into u64.
    assert_ne!(a, b, "two get-random-bytes calls packed to the same u64");
    Ok(())
}
