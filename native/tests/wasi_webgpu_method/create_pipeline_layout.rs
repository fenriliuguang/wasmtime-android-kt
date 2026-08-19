//! L2: `get-device` + `[method]gpu-device.create-pipeline-layout`
//! WIT: `(borrow<gpu-device>, gpu-pipeline-layout-descriptor) -> own<gpu-pipeline-layout>`.
//! Guest passes empty bind-group-layouts + label="l2"; drops own; `run` returns harness 1.

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
struct GpuBindGroupLayout {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuPipelineLayout {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuPipelineLayoutDescriptor {
    #[component(name = "bind-group-layouts")]
    bind_group_layouts: Vec<Option<Resource<GpuBindGroupLayout>>>,
    #[component(name = "immediate-size")]
    immediate_size: Option<u32>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_pipeline_layout(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-bind-group-layout",
        ResourceType::host::<GpuBindGroupLayout>(),
        |mut store, rep| {
            let resource = Resource::<GpuBindGroupLayout>::new_own(rep);
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
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-pipeline-layout",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuPipelineLayoutDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.bind_group_layouts.is_empty(),
                "guest must pass empty bind-group-layouts this slice"
            );
            assert!(descriptor.immediate_size.is_none());
            assert_eq!(descriptor.label.as_deref(), Some("l2"));
            let resource = caller
                .data_mut()
                .table
                .push(GpuPipelineLayout { rep: 61 })?;
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
fn wasi_webgpu_method_create_pipeline_layout_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_pipeline_layout.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_pipeline_layout(&mut linker)?;
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
        "guest run must drop own<gpu-pipeline-layout> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_pipeline_layout_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_pipeline_layout.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_pipeline_layout(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-pipeline-layout> and return harness 1 via call_async"
    );
    Ok(())
}
