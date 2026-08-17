//! S6+: `get-device` + `[method]gpu-device.create-bind-group-layout`
//! WIT: `(borrow<gpu-device>, gpu-bind-group-layout-descriptor) -> own<gpu-bind-group-layout>`.
//! Guest passes empty entries; drops own; `run` returns harness 1.

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

use crate::texture_format::GpuTextureFormat;

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

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureViewDimension {
    #[component(name = "d1")]
    D1,
    #[component(name = "d2")]
    D2,
    #[component(name = "d2-array")]
    D2Array,
    #[component(name = "cube")]
    Cube,
    #[component(name = "cube-array")]
    CubeArray,
    #[component(name = "d3")]
    D3,
}

flags! {
    GpuShaderStage {
        #[component(name = "vertex")]
        const VERTEX;
        #[component(name = "fragment")]
        const FRAGMENT;
        #[component(name = "compute")]
        const COMPUTE;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuBufferBindingType {
    #[component(name = "uniform")]
    Uniform,
    #[component(name = "storage")]
    Storage,
    #[component(name = "read-only-storage")]
    ReadOnlyStorage,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuSamplerBindingType {
    #[component(name = "filtering")]
    Filtering,
    #[component(name = "non-filtering")]
    NonFiltering,
    #[component(name = "comparison")]
    Comparison,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureSampleType {
    #[component(name = "float")]
    Float,
    #[component(name = "unfilterable-float")]
    UnfilterableFloat,
    #[component(name = "depth")]
    Depth,
    #[component(name = "sint")]
    Sint,
    #[component(name = "uint")]
    Uint,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuStorageTextureAccess {
    #[component(name = "write-only")]
    WriteOnly,
    #[component(name = "read-only")]
    ReadOnly,
    #[component(name = "read-write")]
    ReadWrite,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBufferBindingLayout {
    #[component(name = "type")]
    ty: Option<GpuBufferBindingType>,
    #[component(name = "has-dynamic-offset")]
    has_dynamic_offset: Option<bool>,
    #[component(name = "min-binding-size")]
    min_binding_size: Option<u64>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuSamplerBindingLayout {
    #[component(name = "type")]
    ty: Option<GpuSamplerBindingType>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuTextureBindingLayout {
    #[component(name = "sample-type")]
    sample_type: Option<GpuTextureSampleType>,
    #[component(name = "view-dimension")]
    view_dimension: Option<GpuTextureViewDimension>,
    multisampled: Option<bool>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuStorageTextureBindingLayout {
    access: Option<GpuStorageTextureAccess>,
    format: GpuTextureFormat,
    #[component(name = "view-dimension")]
    view_dimension: Option<GpuTextureViewDimension>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuBindGroupLayoutEntry {
    binding: u32,
    visibility: GpuShaderStage,
    buffer: Option<GpuBufferBindingLayout>,
    sampler: Option<GpuSamplerBindingLayout>,
    texture: Option<GpuTextureBindingLayout>,
    #[component(name = "storage-texture")]
    storage_texture: Option<GpuStorageTextureBindingLayout>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuBindGroupLayoutDescriptor {
    entries: Vec<GpuBindGroupLayoutEntry>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_bind_group_layout(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-bind-group-layout",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, GpuBindGroupLayoutDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.entries.is_empty(),
                "guest must pass empty bind-group-layout entries this slice"
            );
            assert!(descriptor.label.is_none());
            let resource = caller
                .data_mut()
                .table
                .push(GpuBindGroupLayout { rep: 59 })?;
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
fn wasi_webgpu_method_create_bind_group_layout_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_bind_group_layout.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_bind_group_layout(&mut linker)?;
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
        "guest run must drop own<gpu-bind-group-layout> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_bind_group_layout_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_bind_group_layout.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_bind_group_layout(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own<gpu-bind-group-layout> and return harness 1 via call_async"
    );
    Ok(())
}
