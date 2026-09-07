//! WASI 0.3: wasi:cli/exit@0.3.0 `exit`.
//! Guest `exit` completes `run` with the official empty `result`.
//! Must not kill the process (`process::exit` / abort). `exit-with-code` is not this lane.

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

/// Same typed unwind the product linker uses. Integration tests cannot see `cm.rs` types.
#[derive(Debug)]
struct CliExit(Result<(), ()>);

impl std::fmt::Display for CliExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Ok(()) => f.write_str("wasi:cli/exit ok"),
            Err(()) => f.write_str("wasi:cli/exit err"),
        }
    }
}

impl std::error::Error for CliExit {}

fn map_cli_run_result(result: wasmtime::Result<u32>) -> wasmtime::Result<u32> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => match find_cli_exit(&e) {
            Some(Ok(())) => Ok(0),
            Some(Err(())) => Ok(1),
            None => Err(e),
        },
    }
}

fn find_cli_exit(err: &wasmtime::Error) -> Option<Result<(), ()>> {
    for cause in err.chain() {
        if let Some(exit) = cause.downcast_ref::<CliExit>() {
            return Some(exit.0);
        }
    }
    None
}

fn flatten_concurrent<T>(result: wasmtime::Result<wasmtime::Result<T>>) -> wasmtime::Result<T> {
    match result {
        Ok(inner) => inner,
        Err(e) => Err(e),
    }
}

fn register(linker: &mut Linker<()>) -> wasmtime::Result<()> {
    linker.instance("wasi:cli/exit@0.3.0")?.func_wrap(
        "exit",
        |_store, (status,): (Result<(), ()>,)| -> wasmtime::Result<()> {
            Err(CliExit(status).into())
        },
    )?;
    Ok(())
}

fn engine() -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    Engine::new(&config)
}

fn load(engine: &Engine, file: &str) -> wasmtime::Result<Component> {
    let path = format!("{}/../fixtures/wasi/{file}", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(path)?;
    Component::new(engine, bytes)
}

fn instantiate(file: &str) -> wasmtime::Result<(Store<()>, wasmtime::component::Instance)> {
    let engine = engine()?;
    let component = load(&engine, file)?;
    let mut linker = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    Ok((store, instance))
}

fn call_run(file: &str) -> wasmtime::Result<u32> {
    let (mut store, instance) = instantiate(file)?;
    let result = pollster::block_on(async {
        flatten_concurrent(
            store
                .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                    let func = accessor.with(|mut access| {
                        instance.get_typed_func::<(), (u32,)>(&mut access, "run")
                    })?;
                    let (value,) = func.call_concurrent(accessor, ()).await?;
                    Ok(value)
                })
                .await,
        )
    });
    map_cli_run_result(result)
}

fn call_run_sync(file: &str) -> wasmtime::Result<u32> {
    let (mut store, instance) = instantiate(file)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    map_cli_run_result(func.call(&mut store, ()).map(|(v,)| v))
}

fn call_official_run(file: &str) -> wasmtime::Result<Result<(), ()>> {
    let (mut store, instance) = instantiate(file)?;
    let result = pollster::block_on(async {
        flatten_concurrent(
            store
                .run_concurrent(async |accessor| -> wasmtime::Result<Result<(), ()>> {
                    let idx = accessor.with(|mut access| {
                        let inst = instance
                            .get_export_index(&mut access, None, "wasi:cli/run@0.3.0")
                            .ok_or_else(|| wasmtime::Error::msg("missing wasi:cli/run@0.3.0"))?;
                        instance
                            .get_export_index(&mut access, Some(&inst), "run")
                            .ok_or_else(|| wasmtime::Error::msg("missing run"))
                    })?;
                    let func = accessor.with(|mut access| {
                        instance.get_typed_func::<(), (Result<(), ()>,)>(&mut access, idx)
                    })?;
                    let (result,) = func.call_concurrent(accessor, ()).await?;
                    Ok(result)
                })
                .await,
        )
    });
    match result {
        Ok(status) => Ok(status),
        Err(e) => match find_cli_exit(&e) {
            Some(status) => Ok(status),
            None => Err(e),
        },
    }
}

#[test]
fn wasi_cli_exit_ok_maps_to_zero() -> wasmtime::Result<()> {
    assert_eq!(call_run("cli_exit.wasm")?, 0, "exit(ok) → harness 0");
    Ok(())
}

#[test]
fn wasi_cli_exit_err_maps_to_one() -> wasmtime::Result<()> {
    assert_eq!(call_run("cli_exit_err.wasm")?, 1, "exit(err) → harness 1");
    Ok(())
}

#[test]
fn wasi_cli_exit_sync_ok_maps_to_zero() -> wasmtime::Result<()> {
    assert_eq!(call_run_sync("cli_exit.wasm")?, 0, "sync exit(ok) → 0");
    Ok(())
}

#[test]
fn wasi_cli_exit_official_run_ok() -> wasmtime::Result<()> {
    assert!(
        call_official_run("cli_exit.wasm")?.is_ok(),
        "exit(ok) completes wasi:cli/run with official ok"
    );
    Ok(())
}

#[test]
fn wasi_cli_exit_official_run_err() -> wasmtime::Result<()> {
    assert!(
        call_official_run("cli_exit_err.wasm")?.is_err(),
        "exit(err) completes wasi:cli/run with official err"
    );
    Ok(())
}

#[test]
fn wasi_cli_exit_does_not_kill_process() -> wasmtime::Result<()> {
    let _ = call_run("cli_exit.wasm")?;
    let _ = call_run("cli_exit_err.wasm")?;
    Ok(())
}
