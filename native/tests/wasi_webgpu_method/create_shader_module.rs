//! L2: `get-device` + `[method]gpu-device.create-shader-module`
//! WIT: `(borrow<gpu-device>, gpu-shader-module-descriptor) -> own<gpu-shader-module>`.
//! Guest passes WGSL `fn l2`; hints/label none; drops own; `run` returns harness 1.

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuDevice {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuPipelineLayout {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuShaderModule {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum GpuLayoutMode {
    #[component(name = "specific")]
    Specific(Resource<GpuPipelineLayout>),
    #[component(name = "auto")]
    Auto,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuShaderModuleCompilationHint {
    #[component(name = "entry-point")]
    entry_point: String,
    layout: Option<GpuLayoutMode>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuShaderModuleDescriptor {
    code: String,
    #[component(name = "compilation-hints")]
    compilation_hints: Option<Vec<GpuShaderModuleCompilationHint>>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_shader_module(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-device",
        ResourceType::host::<GpuDevice>(),
        |mut store, rep| {
            let resource = Resource::<GpuDevice>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-pipeline-layout",
        ResourceType::host::<GpuPipelineLayout>(),
        |mut store, rep| {
            let resource = Resource::<GpuPipelineLayout>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-shader-module",
        ResourceType::host::<GpuShaderModule>(),
        |mut store, rep| {
            let resource = Resource::<GpuShaderModule>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-shader-module",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuShaderModuleDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert_eq!(
                descriptor.code, "@compute @workgroup_size(1) fn l2() {}",
                "guest must pass L2 WGSL this slice"
            );
            assert!(descriptor.compilation_hints.is_none());
            assert!(descriptor.label.is_none());
            let resource = caller.data_mut().table.push(GpuShaderModule { rep: 43 })?;
            Ok((resource,))
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
fn wasi_webgpu_method_create_shader_module_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_shader_module.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_shader_module(&mut linker)?;
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
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-shader-module> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_shader_module_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_shader_module.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_shader_module(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-shader-module> and return harness 1 via call_async"
    );
    Ok(())
}
