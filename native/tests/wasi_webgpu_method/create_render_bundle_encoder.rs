//! S6+: `get-device` + `[method]gpu-device.create-render-bundle-encoder`
//! WIT: `(borrow<gpu-device>, gpu-render-bundle-encoder-descriptor)
//!      -> own<gpu-render-bundle-encoder>`.
//! Guest passes empty color-formats and none options; drops own; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

use crate::texture_format::GpuTextureFormat;

#[derive(Debug)]
struct GpuDevice;

#[derive(Debug)]
struct GpuRenderBundleEncoder;

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRenderBundleEncoderDescriptor {
    #[component(name = "depth-read-only")]
    depth_read_only: Option<bool>,
    #[component(name = "stencil-read-only")]
    stencil_read_only: Option<bool>,
    #[component(name = "color-formats")]
    color_formats: Vec<Option<GpuTextureFormat>>,
    #[component(name = "depth-stencil-format")]
    depth_stencil_format: Option<GpuTextureFormat>,
    #[component(name = "sample-count")]
    sample_count: Option<u32>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>, called: Arc<AtomicBool>) -> wasmtime::Result<()> {
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
        "gpu-render-bundle-encoder",
        ResourceType::host::<GpuRenderBundleEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderBundleEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-device.create-render-bundle-encoder",
        move |mut caller,
              (device, descriptor): (Resource<GpuDevice>, GpuRenderBundleEncoderDescriptor)| {
            caller.data_mut().table.get(&device).map(|_| ())?;
            assert!(descriptor.depth_read_only.is_none());
            assert!(descriptor.stencil_read_only.is_none());
            assert!(descriptor.color_formats.is_empty());
            assert!(descriptor.depth_stencil_format.is_none());
            assert!(descriptor.sample_count.is_none());
            assert!(descriptor.label.is_none());
            called.store(true, Ordering::SeqCst);
            let resource = caller.data_mut().table.push(GpuRenderBundleEncoder)?;
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
fn wasi_webgpu_method_create_render_bundle_encoder_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_render_bundle_encoder.wasm"
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
fn wasi_webgpu_method_create_render_bundle_encoder_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_create_render_bundle_encoder.wasm"
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
