//! S6+: `get-canvas-context` + `[method]gpu-canvas-context.get-configuration`
//! WIT: `(borrow) -> option<gpu-canvas-configuration-owned>`. Lift-only none; harness 1.

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

use crate::texture_format::GpuTextureFormat;

#[derive(Debug)]
struct GpuDevice;

#[derive(Debug)]
struct GpuCanvasContext;

flags! {
    GpuTextureUsage {
        #[component(name = "copy-src")]
        const COPY_SRC;
        #[component(name = "copy-dst")]
        const COPY_DST;
        #[component(name = "texture-binding")]
        const TEXTURE_BINDING;
        #[component(name = "storage-binding")]
        const STORAGE_BINDING;
        #[component(name = "render-attachment")]
        const RENDER_ATTACHMENT;
        #[component(name = "transient-attachment")]
        const TRANSIENT_ATTACHMENT;
    }
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum PredefinedColorSpace {
    #[component(name = "srgb")]
    Srgb,
    #[component(name = "display-p3")]
    DisplayP3,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCanvasAlphaMode {
    #[component(name = "opaque")]
    Opaque,
    #[component(name = "premultiplied")]
    Premultiplied,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuCanvasToneMappingMode {
    #[component(name = "standard")]
    Standard,
    #[component(name = "extended")]
    Extended,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuCanvasToneMapping {
    mode: Option<GpuCanvasToneMappingMode>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuCanvasConfigurationOwned {
    device: Resource<GpuDevice>,
    format: GpuTextureFormat,
    usage: Option<GpuTextureUsage>,
    #[component(name = "view-formats")]
    view_formats: Option<Vec<GpuTextureFormat>>,
    #[component(name = "color-space")]
    color_space: Option<PredefinedColorSpace>,
    #[component(name = "tone-mapping")]
    tone_mapping: Option<GpuCanvasToneMapping>,
    #[component(name = "alpha-mode")]
    alpha_mode: Option<GpuCanvasAlphaMode>,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-canvas-context",
        ResourceType::host::<GpuCanvasContext>(),
        |mut store, rep| {
            let resource = Resource::<GpuCanvasContext>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-canvas-context", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCanvasContext)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-canvas-context.get-configuration",
        |mut caller, (ctx,): (Resource<GpuCanvasContext>,)| {
            caller.data_mut().table.get(&ctx).map(|_| ())?;
            Ok((Option::<GpuCanvasConfigurationOwned>::None,))
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
fn wasi_webgpu_method_canvas_context_get_configuration_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_canvas_context_get_configuration.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker)?;
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
    Ok(())
}

#[test]
fn wasi_webgpu_method_canvas_context_get_configuration_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_canvas_context_get_configuration.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1);
    Ok(())
}
