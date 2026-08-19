//! L2: `get-device` + `get-bind-group-layout` + `[method]gpu-device.create-bind-group`
//! WIT: `(borrow<gpu-device>, gpu-bind-group-descriptor) -> own<gpu-bind-group>`.
//! Guest passes layout borrow + empty entries + label="l2"; drops own; `run` returns harness 1.

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
struct GpuBindGroup {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuBuffer {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuSampler {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTexture {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTextureView {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBufferBinding {
    buffer: Resource<GpuBuffer>,
    offset: Option<u64>,
    size: Option<u64>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum GpuBindingResource {
    #[component(name = "gpu-buffer")]
    GpuBuffer(Resource<GpuBuffer>),
    #[component(name = "gpu-buffer-binding")]
    GpuBufferBinding(GpuBufferBinding),
    #[component(name = "gpu-sampler")]
    GpuSampler(Resource<GpuSampler>),
    #[component(name = "gpu-texture")]
    GpuTexture(Resource<GpuTexture>),
    #[component(name = "gpu-texture-view")]
    GpuTextureView(Resource<GpuTextureView>),
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBindGroupEntry {
    binding: u32,
    resource: GpuBindingResource,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBindGroupDescriptor {
    layout: Resource<GpuBindGroupLayout>,
    entries: Vec<GpuBindGroupEntry>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_bind_group(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-bind-group",
        ResourceType::host::<GpuBindGroup>(),
        |mut store, rep| {
            let resource = Resource::<GpuBindGroup>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-buffer",
        ResourceType::host::<GpuBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-sampler",
        ResourceType::host::<GpuSampler>(),
        |mut store, rep| {
            let resource = Resource::<GpuSampler>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-texture",
        ResourceType::host::<GpuTexture>(),
        |mut store, rep| {
            let resource = Resource::<GpuTexture>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-texture-view",
        ResourceType::host::<GpuTextureView>(),
        |mut store, rep| {
            let resource = Resource::<GpuTextureView>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-bind-group-layout", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBindGroupLayout { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-bind-group",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBindGroupDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&descriptor.layout)
                .map(|_| ())?;
            assert!(
                descriptor.entries.is_empty(),
                "guest must pass empty bind-group entries this slice"
            );
            assert_eq!(descriptor.label.as_deref(), Some("l2"));
            let resource = caller.data_mut().table.push(GpuBindGroup { rep: 67 })?;
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
fn wasi_webgpu_method_create_bind_group_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_bind_group.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_bind_group(&mut linker)?;
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
        "guest run must drop own<gpu-bind-group> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_bind_group_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_bind_group.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_bind_group(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-bind-group> and return harness 1 via call_async"
    );
    Ok(())
}
