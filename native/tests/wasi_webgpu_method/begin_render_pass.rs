//! W3: `get-encoder` + `[method]gpu-command-encoder.begin-render-pass`
//! (resource self, sync; stub view 23 → pass 29).

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuCommandEncoder;

struct TestHost {
    table: ResourceTable,
}

fn register_method_begin_render_pass(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-command-encoder",
        ResourceType::host::<GpuCommandEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuCommandEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCommandEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.begin-render-pass",
        |mut caller, (encoder, view): (Resource<GpuCommandEncoder>, u32)| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert_eq!(view, 23, "guest must pass stub view 23");
            Ok((29u32,))
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
fn wasi_webgpu_method_begin_render_pass_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_begin_render_pass.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_begin_render_pass(&mut linker)?;

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
    assert_eq!(v, 29, "guest run must return stub pass rep via [method]");
    Ok(())
}

#[test]
fn wasi_webgpu_method_begin_render_pass_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_begin_render_pass.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;

    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_begin_render_pass(&mut linker)?;

    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 29,
        "guest run must return stub pass rep via [method] call_async"
    );
    Ok(())
}
