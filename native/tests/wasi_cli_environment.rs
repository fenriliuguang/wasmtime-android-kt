//! WASI 0.3: wasi:cli/environment@0.3.0 get-environment / get-arguments.
//! Host supplies TMPDIR only (Android: empty or documented TMPDIR). Arguments empty.

use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

fn cli_environment(tmpdir: Option<&str>) -> Vec<(String, String)> {
    match tmpdir {
        Some(v) => vec![("TMPDIR".to_string(), v.to_string())],
        None => Vec::new(),
    }
}

fn register(linker: &mut Linker<()>, tmpdir: Option<String>) -> wasmtime::Result<()> {
    let mut environment = linker.instance("wasi:cli/environment@0.3.0")?;
    let tmpdir_env = tmpdir.clone();
    environment.func_wrap("get-environment", move |_store, ()| {
        Ok((cli_environment(tmpdir_env.as_deref()),))
    })?;
    environment.func_wrap("get-arguments", |_store, ()| Ok((Vec::<String>::new(),)))?;
    Ok(())
}

fn call_run(tmpdir: Option<&str>) -> wasmtime::Result<u32> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/cli_environment.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker = Linker::new(&engine);
    register(&mut linker, tmpdir.map(str::to_string))?;
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = func.call(&mut store, ())?;
    Ok(v)
}

#[test]
fn wasi_cli_environment_tmpdir_and_empty_args() -> wasmtime::Result<()> {
    assert_eq!(
        call_run(Some("/tmp/p3env"))?,
        1,
        "guest must see TMPDIR=/tmp/p3env and empty arguments"
    );
    Ok(())
}

#[test]
fn wasi_cli_environment_empty_without_tmpdir() -> wasmtime::Result<()> {
    assert_eq!(
        call_run(None)?,
        0,
        "without TMPDIR the environment list is empty"
    );
    Ok(())
}
