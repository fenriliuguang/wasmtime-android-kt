//! WASI 0.3: wasi:clocks/system-clock@0.3.0#now smoke (transitional u64 unix seconds).

use std::time::{SystemTime, UNIX_EPOCH};

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

#[test]
fn wasi_system_clock_now_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/system_now.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:clocks/system-clock@0.3.0")?
        .func_wrap("now", |_store, ()| {
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            Ok((secs,))
        })?;

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u64,)>(&mut store, "run")?;
    let (secs,) = func.call(&mut store, ())?;
    // After 2024-01-01 and before year ~2100 — proves wall-clock, not monotonic epoch.
    assert!(
        secs > 1_704_067_200 && secs < 4_102_444_800,
        "system-clock seconds out of expected unix range: {secs}"
    );
    Ok(())
}
