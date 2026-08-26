//! WASI 0.3: wasi:clocks/system-clock@0.3.0#now smoke (official instant record).

use std::time::{SystemTime, UNIX_EPOCH};

use wasmtime::component::{Component, ComponentType, Lift, Linker, Lower};
use wasmtime::{Config, Engine, Store};

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct Instant {
    seconds: i64,
    nanoseconds: u32,
}

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
            let d = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            Ok((Instant {
                seconds: d.as_secs() as i64,
                nanoseconds: d.subsec_nanos(),
            },))
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
