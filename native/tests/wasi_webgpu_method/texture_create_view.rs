//! W3+: `get-texture` + `[method]gpu-texture.create-view`. Stub returns 41.

use wasmtime::component::{Component, Linker, Resource, ResourceTable, ResourceType};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuTexture;

struct TestHost {
    table: ResourceTable,
}

fn register_method_texture_create_view(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-texture",
        ResourceType::host::<GpuTexture>(),
        |mut store, rep| {
            let resource = Resource::<GpuTexture>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-texture", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuTexture)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-texture.create-view",
        |mut caller, (texture,): (Resource<GpuTexture>,)| {
            caller.data_mut().table.get(&texture).map(|_| ())?;
            Ok((41u32,))
        },
    )?;
    Ok(())
}

fn new_store(engine: &Engine) -> Store<TestHost> {
    Store::new(engine, TestHost { table: ResourceTable::new() })
}

#[test]
fn wasi_webgpu_method_texture_create_view_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_texture_create_view.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_texture_create_view(&mut linker)?;
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
    assert_eq!(v, 41, "guest run must return stub view rep via [method]");
    Ok(())
}

#[test]
fn wasi_webgpu_method_texture_create_view_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_texture_create_view.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_texture_create_view(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 41, "guest run must return stub view rep via [method] call_async");
    Ok(())
}
