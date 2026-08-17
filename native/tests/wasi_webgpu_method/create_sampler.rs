//! S8: `get-device` + `[method]gpu-device.create-sampler`
//! WIT: `(borrow<gpu-device>, option<gpu-sampler-descriptor>) -> own<gpu-sampler>`.
//! Guest passes none; drops own; `run` returns harness 1.

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
struct GpuSampler {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuAddressMode {
    #[component(name = "clamp-to-edge")]
    ClampToEdge,
    #[component(name = "repeat")]
    Repeat,
    #[component(name = "mirror-repeat")]
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuFilterMode {
    #[component(name = "nearest")]
    Nearest,
    #[component(name = "linear")]
    Linear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuMipmapFilterMode {
    #[component(name = "nearest")]
    Nearest,
    #[component(name = "linear")]
    Linear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCompareFunction {
    #[component(name = "never")]
    Never,
    #[component(name = "less")]
    Less,
    #[component(name = "equal")]
    Equal,
    #[component(name = "less-equal")]
    LessEqual,
    #[component(name = "greater")]
    Greater,
    #[component(name = "not-equal")]
    NotEqual,
    #[component(name = "greater-equal")]
    GreaterEqual,
    #[component(name = "always")]
    Always,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuSamplerDescriptor {
    #[component(name = "address-mode-u")]
    address_mode_u: Option<GpuAddressMode>,
    #[component(name = "address-mode-v")]
    address_mode_v: Option<GpuAddressMode>,
    #[component(name = "address-mode-w")]
    address_mode_w: Option<GpuAddressMode>,
    #[component(name = "mag-filter")]
    mag_filter: Option<GpuFilterMode>,
    #[component(name = "min-filter")]
    min_filter: Option<GpuFilterMode>,
    #[component(name = "mipmap-filter")]
    mipmap_filter: Option<GpuMipmapFilterMode>,
    #[component(name = "lod-min-clamp")]
    lod_min_clamp: Option<f32>,
    #[component(name = "lod-max-clamp")]
    lod_max_clamp: Option<f32>,
    compare: Option<GpuCompareFunction>,
    #[component(name = "max-anisotropy")]
    max_anisotropy: Option<u16>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_create_sampler(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-sampler",
        ResourceType::host::<GpuSampler>(),
        |mut store, rep| {
            let resource = Resource::<GpuSampler>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-sampler",
        |mut caller, (device, descriptor): (Resource<GpuDevice>, Option<GpuSamplerDescriptor>)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(
                descriptor.is_none(),
                "guest must pass descriptor=none this slice"
            );
            let resource = caller.data_mut().table.push(GpuSampler { rep: 53 })?;
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
fn wasi_webgpu_method_create_sampler_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_sampler.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_sampler(&mut linker)?;
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
    assert_eq!(v, 1, "guest run must drop owns and return harness 1");
    Ok(())
}

#[test]
fn wasi_webgpu_method_create_sampler_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_sampler.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_create_sampler(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop owns and return harness 1 via call_async"
    );
    Ok(())
}
