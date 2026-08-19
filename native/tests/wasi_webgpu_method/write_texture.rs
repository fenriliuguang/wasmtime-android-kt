//! L2: `get-queue` + `get-texture` + `[method]gpu-queue.write-texture-with-copy`
//! WIT: `(borrow queue, gpu-texel-copy-texture-info, list<u8>,
//!      gpu-texel-copy-buffer-layout, gpu-extent3-d)`.
//! Guest passes texture borrow, 4-byte data `l2\0\0`, bytes-per-row=4, size 1×1×1;
//! `run` returns harness 1.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuQueue;

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
struct GpuTexelCopyTextureInfo {
    texture: Resource<GpuTexture>,
    #[component(name = "mip-level")]
    mip_level: Option<u32>,
    origin: Option<GpuOrigin3D>,
    aspect: Option<GpuTextureAspect>,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuTexelCopyBufferLayout {
    offset: Option<u64>,
    #[component(name = "bytes-per-row")]
    bytes_per_row: Option<u32>,
    #[component(name = "rows-per-image")]
    rows_per_image: Option<u32>,
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

fn register_method_write_texture(
    linker: &mut Linker<TestHost>,
    written: Arc<AtomicBool>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource(
        "gpu-queue",
        ResourceType::host::<GpuQueue>(),
        |mut store, rep| {
            let resource = Resource::<GpuQueue>::new_own(rep);
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
    webgpu.func_wrap("get-queue", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuQueue)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-texture", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuTexture)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-queue.write-texture-with-copy",
        move |mut caller,
              (queue, destination, data, layout, size): (
            Resource<GpuQueue>,
            GpuTexelCopyTextureInfo,
            Vec<u8>,
            GpuTexelCopyBufferLayout,
            GpuExtent3D,
        )| {
            caller.data_mut().table.get(&queue).map(|_| ())?;
            caller
                .data_mut()
                .table
                .get(&destination.texture)
                .map(|_| ())?;
            assert!(
                destination.mip_level.is_none(),
                "guest must pass mip-level=none this slice"
            );
            assert!(
                destination.origin.is_none(),
                "guest must pass origin=none this slice"
            );
            assert!(
                destination.aspect.is_none(),
                "guest must pass aspect=none this slice"
            );
            assert_eq!(data, b"l2\0\0", "guest must pass 4-byte data l2\\0\\0");
            assert!(
                layout.offset.is_none() && layout.rows_per_image.is_none(),
                "guest must pass layout offset/rows none this slice"
            );
            assert_eq!(
                layout.bytes_per_row,
                Some(4),
                "guest must pass bytes-per-row=some(4)"
            );
            assert_eq!(size.width, 1, "guest must pass width=1");
            assert_eq!(size.height, Some(1), "guest must pass height=some(1)");
            assert_eq!(
                size.depth_or_array_layers,
                Some(1),
                "guest must pass depth=some(1)"
            );
            written.store(true, Ordering::SeqCst);
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
fn wasi_webgpu_method_write_texture_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_write_texture.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let written = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_write_texture(&mut linker, written.clone())?;
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
    assert_eq!(v, 1, "guest run must return harness 1 after write-texture");
    assert!(
        written.load(Ordering::SeqCst),
        "write-texture-with-copy must have been called"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_write_texture_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_write_texture.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let written = Arc::new(AtomicBool::new(false));
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_write_texture(&mut linker, written.clone())?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(v, 1, "guest run must return harness 1 via call_async");
    assert!(
        written.load(Ordering::SeqCst),
        "write-texture-with-copy must have been called"
    );
    Ok(())
}
