//! WASI 0.3: wasi:clocks/system-clock@0.3.0#resolution smoke.
//! Transitional `func() -> u64` nanoseconds (official WIT may be a datetime record).

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn wasi_system_clock_resolution_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/system_resolution.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:clocks/system-clock@0.3.0")?
        .func_wrap("resolution", |_store, ()| {
            Ok((1u64,))
        })?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u64,)>(&mut store, "run")?;
    let (resolution,) = func.call(&mut store, ())?;
    assert_eq!(resolution, 1, "host resolution is 1 ns: got {resolution}");
    Ok(())
}
