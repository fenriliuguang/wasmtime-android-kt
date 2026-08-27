//! L2: `get-compilation-message` + `[method]gpu-compilation-message.offset`
//! WIT: `(borrow) -> u64`. Host returns Cpu stub 100; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuCompilationMessage {
    shader_module: u32,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>, called: Arc<AtomicBool>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-compilation-message",
        ResourceType::host::<GpuCompilationMessage>(),
        |mut store, rep| {
            let resource = Resource::<GpuCompilationMessage>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-compilation-message", |mut store, ()| {
        let resource = store
            .data_mut()
            .table
            .push(GpuCompilationMessage { shader_module: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-compilation-message.offset",
        move |mut caller, (msg,): (Resource<GpuCompilationMessage>,)| {
            caller.data_mut().table.get(&msg).map(|_| ())?;
            called.store(true, Ordering::SeqCst);
            Ok((100u64,))
        },
    )?;
    Ok(())
}

fn new_store(engine: &Engine) -> Store<TestHost> {
    Store::new(
        engine,
        TestHost {
            table: ResourceTable::new(),
        },
    )
}

#[test]
fn wasi_webgpu_method_compilation_message_offset_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_compilation_message_offset.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker, called.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let v = pollster::block_on(async {
        store
            .run_concurrent(async |accessor| -> wasmtime::Result<u32> {
                let func = accessor
                    .with(|mut access| instance.get_typed_func::<(), (u32,)>(&mut access, "run"))?;
                let (value,) = func.call_concurrent(accessor, ()).await?;
                Ok(value)
            })
            .await?
    })?;
    assert_eq!(v, 1);
    assert!(called.load(Ordering::SeqCst));
    Ok(())
}

#[test]
fn wasi_webgpu_method_compilation_message_offset_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_compilation_message_offset.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker, called.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1);
    assert!(called.load(Ordering::SeqCst));
    Ok(())
}
