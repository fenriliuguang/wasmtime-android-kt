//! WASI 0.3: wasi:clocks/monotonic-clock@0.3.0#now smoke.

use std::sync::OnceLock;
use std::time::Instant;

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn wasi_monotonic_clock_now_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/monotonic_now.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:clocks/monotonic-clock@0.3.0")?
        .func_wrap("now", |_store, ()| {
            static START: OnceLock<Instant> = OnceLock::new();
            let start = START.get_or_init(Instant::now);
            Ok((start.elapsed().as_nanos() as u64,))
        })?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u64,)>(&mut store, "run")?;
    let (a,) = func.call(&mut store, ())?;
    let (b,) = func.call(&mut store, ())?;
    assert!(b >= a, "monotonic mark must not decrease: first={a} second={b}");
    Ok(())
}
