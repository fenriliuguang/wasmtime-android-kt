//! L2: `get-encoder` + `get-buffer` + `get-texture` +
//! `[method]gpu-command-encoder.copy-buffer-to-texture`
//! WIT: `(borrow encoder, gpu-texel-copy-buffer-info,
//!      gpu-texel-copy-texture-info, gpu-extent3-d)`.
//! Guest passes layout/mip/origin/aspect none, size 1×1×1; harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuCommandEncoder;

#[derive(Debug)]
struct GpuBuffer;

#[derive(Debug)]
struct GpuTexture;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuTextureAspect {
    #[component(name = "all")]
    All,
    #[component(name = "stencil-only")]
    StencilOnly,
    #[component(name = "depth-only")]
    DepthOnly,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuOrigin3D {
    x: Option<u32>,
    y: Option<u32>,
    z: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuTexelCopyBufferInfo {
    buffer: Resource<GpuBuffer>,
    offset: Option<u64>,
    #[component(name = "bytes-per-row")]
    bytes_per_row: Option<u32>,
    #[component(name = "rows-per-image")]
    rows_per_image: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuTexelCopyTextureInfo {
    texture: Resource<GpuTexture>,
    #[component(name = "mip-level")]
    mip_level: Option<u32>,
    origin: Option<GpuOrigin3D>,
    aspect: Option<GpuTextureAspect>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuExtent3D {
    width: u32,
    height: Option<u32>,
    #[component(name = "depth-or-array-layers")]
    depth_or_array_layers: Option<u32>,
}

struct TestHost {
    table: ResourceTable,
}

fn register(linker: &mut Linker<TestHost>, called: Arc<AtomicBool>) -> wasmtime::Result<()> {
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
        "gpu-texture",
        ResourceType::host::<GpuTexture>(),
        |mut store, rep| {
            let resource = Resource::<GpuTexture>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCommandEncoder)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-buffer", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuBuffer)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-texture", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuTexture)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.copy-buffer-to-texture",
        move |mut caller,
              (encoder, source, destination, size): (
            Resource<GpuCommandEncoder>,
            GpuTexelCopyBufferInfo,
            GpuTexelCopyTextureInfo,
            GpuExtent3D,
        )| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            caller.data_mut().table.get(&source.buffer).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&destination.texture)
                .map(|_| ())?;
            assert!(source.offset.is_none());
            assert!(source.bytes_per_row.is_none());
            assert!(source.rows_per_image.is_none());
            assert!(destination.mip_level.is_none());
            assert!(destination.origin.is_none());
            assert!(destination.aspect.is_none());
            assert_eq!(size.width, 1);
            assert_eq!(size.height, Some(1));
            assert_eq!(size.depth_or_array_layers, Some(1));
            called.store(true, Ordering::SeqCst);
            Ok(())
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
fn wasi_webgpu_method_copy_buffer_to_texture_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_copy_buffer_to_texture.wasm"
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
    assert_eq!(
        v, 1,
        "guest run must return harness 1 after copy-buffer-to-texture"
    );
    assert!(
        called.load(Ordering::SeqCst),
        "copy-buffer-to-texture must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_copy_buffer_to_texture_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_copy_buffer_to_texture.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let called = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register(&mut linker, called.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        called.load(Ordering::SeqCst),
        "copy-buffer-to-texture must have been called"
    );
    Ok(())
}
