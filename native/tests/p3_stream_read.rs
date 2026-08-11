//! P3-PRIM-3: host StreamReader (Vec<u8>) → guest canon stream.read.

use wasmtime::component::{Component, Linker, StreamReader};
use wasmtime::{Config, Engine, Store};

const PAYLOAD: &[u8] = b"P3ST";
/// Wasmtime stream.read packed result: (nbytes << 4) | DROPPED(1)
const EXPECTED: u32 = (4 << 4) | 1;

#[test]
fn p3_stream_read_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/p3/stream_read.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let linker: Linker<()> = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(StreamReader<u8>, u32), (u32,)>(&mut store, "read")?;

    let reader = StreamReader::new(&mut store, PAYLOAD.to_vec())?;
    let (packed,) = pollster::block_on(func.call_async(&mut store, (reader, 100)))?;
    assert_eq!(packed, EXPECTED);
    Ok(())
}
