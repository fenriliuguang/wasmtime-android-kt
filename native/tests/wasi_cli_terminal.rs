//! WASI 0.3: wasi:cli/terminal-{stdin,stdout,stderr}@0.3.0.
//! Android: `none` is allowed. Not a fake TTY.

use wasmtime::component::{Component, Linker, Resource, ResourceType};
use wasmtime::{Config, Engine, Store};

struct TerminalInput;
struct TerminalOutput;

fn register(linker: &mut Linker<()>) -> wasmtime::Result<()> {
    linker.instance("wasi:cli/terminal-input@0.3.0")?.resource(
        "terminal-input",
        ResourceType::host::<TerminalInput>(),
        |_, _| Ok(()),
    )?;
    linker
        .instance("wasi:cli/terminal-output@0.3.0")?
        .resource(
            "terminal-output",
            ResourceType::host::<TerminalOutput>(),
            |_, _| Ok(()),
        )?;
    {
        let mut stdin = linker.instance("wasi:cli/terminal-stdin@0.3.0")?;
        stdin.resource(
            "terminal-input",
            ResourceType::host::<TerminalInput>(),
            |_, _| Ok(()),
        )?;
        stdin.func_wrap("get-terminal-stdin", |_store, ()| {
            Ok((Option::<Resource<TerminalInput>>::None,))
        })?;
    }
    {
        let mut stdout = linker.instance("wasi:cli/terminal-stdout@0.3.0")?;
        stdout.resource(
            "terminal-output",
            ResourceType::host::<TerminalOutput>(),
            |_, _| Ok(()),
        )?;
        stdout.func_wrap("get-terminal-stdout", |_store, ()| {
            Ok((Option::<Resource<TerminalOutput>>::None,))
        })?;
    }
    {
        let mut stderr = linker.instance("wasi:cli/terminal-stderr@0.3.0")?;
        stderr.resource(
            "terminal-output",
            ResourceType::host::<TerminalOutput>(),
            |_, _| Ok(()),
        )?;
        stderr.func_wrap("get-terminal-stderr", |_store, ()| {
            Ok((Option::<Resource<TerminalOutput>>::None,))
        })?;
    }
    Ok(())
}

fn call_run() -> wasmtime::Result<u32> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/cli_terminal.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = func.call(&mut store, ())?;
    Ok(v)
}

#[test]
fn wasi_cli_terminal_all_none() -> wasmtime::Result<()> {
    assert_eq!(
        call_run()?,
        1,
        "Android terminal-stdin/stdout/stderr must be none (not a fake TTY)"
    );
    Ok(())
}
