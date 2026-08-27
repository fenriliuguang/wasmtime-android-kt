//! WASI 0.3: wasi:cli/stdin@0.3.0#read-via-stream smoke (official tuple + result ok).

use wasmtime::component::{
    Component, ComponentType, FutureReader, Lift, Linker, Lower, StreamReader,
};
use wasmtime::{Config, Engine, Store};

const PAYLOAD: &[u8] = b"IN\n";

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum CliErrorCode {
    #[component(name = "unknown")]
    Unknown,
    #[component(name = "io")]
    Io,
    #[component(name = "illegal-byte-sequence")]
    IllegalByteSequence,
    #[component(name = "pipe")]
    Pipe,
}

#[test]
fn wasi_cli_stdin_read_via_stream_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/cli_stdin.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker = Linker::new(&engine);
    linker
        .instance("wasi:cli/stdin@0.3.0")?
        .func_wrap("read-via-stream", |mut store, ()| {
            let reader = StreamReader::new(&mut store, PAYLOAD.to_vec())?;
            let fut = FutureReader::new(&mut store, async move {
                Ok::<_, wasmtime::Error>(Ok::<(), CliErrorCode>(()))
            })?;
            Ok(((reader, fut),))
        })?;

    let mut store = Store::new(&engine, ());
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (n,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(n, PAYLOAD.len() as u32);
    Ok(())
}
