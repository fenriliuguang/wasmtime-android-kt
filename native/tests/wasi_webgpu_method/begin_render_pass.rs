//! S6+: `get-encoder` + `[method]gpu-command-encoder.begin-render-pass`
//! WIT: `(borrow<gpu-command-encoder>, gpu-render-pass-descriptor)
//!      -> own<gpu-render-pass-encoder>`.
//! Guest passes empty color-attachments and option fields none; drops own;
//! `run` returns harness 1.

use wasmtime::component::{
    Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
};
use wasmtime::{Config, Engine, Store};

#[derive(Debug)]
struct GpuCommandEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuRenderPassEncoder {
    #[allow(dead_code)]
    rep: u32,
}

#[derive(Debug)]
struct GpuTextureView;

#[derive(Debug)]
struct GpuQuerySet;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuLoadOp {
    #[component(name = "load")]
    Load,
    #[component(name = "clear")]
    Clear,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuStoreOp {
    #[component(name = "store")]
    Store,
    #[component(name = "discard")]
    Discard,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassColorAttachment {
    view: Resource<GpuTextureView>,
    #[component(name = "depth-slice")]
    depth_slice: Option<u32>,
    #[component(name = "resolve-target")]
    resolve_target: Option<Resource<GpuTextureView>>,
    #[component(name = "clear-value")]
    clear_value: Option<GpuColor>,
    #[component(name = "load-op")]
    load_op: GpuLoadOp,
    #[component(name = "store-op")]
    store_op: GpuStoreOp,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassDepthStencilAttachment {
    view: Resource<GpuTextureView>,
    #[component(name = "depth-clear-value")]
    depth_clear_value: Option<f32>,
    #[component(name = "depth-load-op")]
    depth_load_op: Option<GpuLoadOp>,
    #[component(name = "depth-store-op")]
    depth_store_op: Option<GpuStoreOp>,
    #[component(name = "depth-read-only")]
    depth_read_only: Option<bool>,
    #[component(name = "stencil-clear-value")]
    stencil_clear_value: Option<u32>,
    #[component(name = "stencil-load-op")]
    stencil_load_op: Option<GpuLoadOp>,
    #[component(name = "stencil-store-op")]
    stencil_store_op: Option<GpuStoreOp>,
    #[component(name = "stencil-read-only")]
    stencil_read_only: Option<bool>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuRenderPassTimestampWrites {
    #[component(name = "query-set")]
    query_set: Resource<GpuQuerySet>,
    #[component(name = "beginning-of-pass-write-index")]
    beginning_of_pass_write_index: Option<u32>,
    #[component(name = "end-of-pass-write-index")]
    end_of_pass_write_index: Option<u32>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRenderPassDescriptor {
    #[component(name = "color-attachments")]
    color_attachments: Vec<Option<GpuRenderPassColorAttachment>>,
    #[component(name = "depth-stencil-attachment")]
    depth_stencil_attachment: Option<GpuRenderPassDepthStencilAttachment>,
    #[component(name = "occlusion-query-set")]
    occlusion_query_set: Option<Resource<GpuQuerySet>>,
    #[component(name = "timestamp-writes")]
    timestamp_writes: Option<GpuRenderPassTimestampWrites>,
    #[component(name = "max-draw-count")]
    max_draw_count: Option<u64>,
    label: Option<String>,
}

struct TestHost {
    table: ResourceTable,
}

fn register_method_begin_render_pass(linker: &mut Linker<TestHost>) -> wasmtime::Result<()> {
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
        "gpu-texture-view",
        ResourceType::host::<GpuTextureView>(),
        |mut store, rep| {
            let resource = Resource::<GpuTextureView>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-query-set",
        ResourceType::host::<GpuQuerySet>(),
        |mut store, rep| {
            let resource = Resource::<GpuQuerySet>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "gpu-render-pass-encoder",
        ResourceType::host::<GpuRenderPassEncoder>(),
        |mut store, rep| {
            let resource = Resource::<GpuRenderPassEncoder>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-encoder", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCommandEncoder { rep: 0 })?;
        Ok((resource,))
    })?;
    webgpu.func_wrap(
        "[method]gpu-command-encoder.begin-render-pass",
        |mut caller,
         (encoder, descriptor): (Resource<GpuCommandEncoder>, GpuRenderPassDescriptor)| {
            caller.data_mut().table.get(&encoder).map(|_| ())?;
            assert!(
                descriptor.color_attachments.is_empty(),
                "guest must pass empty color-attachments this slice"
            );
            assert!(descriptor.depth_stencil_attachment.is_none());
            assert!(descriptor.occlusion_query_set.is_none());
            assert!(descriptor.timestamp_writes.is_none());
            assert!(descriptor.max_draw_count.is_none());
            assert!(descriptor.label.is_none());
            let resource = caller
                .data_mut()
                .table
                .push(GpuRenderPassEncoder { rep: 29 })?;
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
fn wasi_webgpu_method_begin_render_pass_smoke() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_begin_render_pass.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_begin_render_pass(&mut linker)?;
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
        "guest run must drop own<gpu-render-pass-encoder> and return harness 1"
    );
    Ok(())
}

#[test]
fn wasi_webgpu_method_begin_render_pass_call_async() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/w1/webgpu_method_begin_render_pass.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker: Linker<TestHost> = Linker::new(&engine);
    register_method_begin_render_pass(&mut linker)?;
    let mut store = new_store(&engine);
    let instance = pollster::block_on(linker.instantiate_async(&mut store, &component))?;
    let func = instance.get_typed_func::<(), (u32,)>(&mut store, "run")?;
    let (v,) = pollster::block_on(func.call_async(&mut store, ()))?;
    assert_eq!(
        v, 1,
        "guest run must drop own pass and return harness 1 via call_async"
    );
    Ok(())
}
