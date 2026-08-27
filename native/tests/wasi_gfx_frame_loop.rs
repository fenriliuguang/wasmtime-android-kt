//! P010-GFXL: product guest `on-frame` loop → get-current-texture → submit →
//! present. Two `frame-event`s produced on a helper thread named `GpuThread`.
//! No JS-style callback. Device multi-frame is the smoke-app instrument.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use wasmtime::component::{
    flags, Component, ComponentType, Lift, Linker, Lower, Resource, ResourceTable, ResourceType,
    StreamReader,
};
use wasmtime::{Config, Engine, Store};

#[path = "wasi_webgpu_method/texture_format.rs"]
mod texture_format;
use texture_format::GpuTextureFormat;

static WROTE_ON_GPU_THREAD: AtomicBool = AtomicBool::new(false);

struct GfxSurface;
struct GfxWebGpuContext;
struct GpuDevice;
struct GpuTexture;
struct GpuQueue;
struct GpuCommandBuffer;

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxSurfaceCreateDesc {
    height: Option<u32>,
    width: Option<u32>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GfxFrameEvent {
    nothing: bool,
}

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
struct GpuCanvasConfiguration {
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

fn gfx_on_frame_events(n: usize) -> wasmtime::Result<Vec<GfxFrameEvent>> {
    let (tx, rx) = mpsc::sync_channel(n.max(1));
    thread::Builder::new()
        .name("GpuThread".into())
        .spawn(move || {
            WROTE_ON_GPU_THREAD.store(
                thread::current().name() == Some("GpuThread"),
                Ordering::SeqCst,
            );
            for _ in 0..n {
                let _ = tx.send(GfxFrameEvent { nothing: true });
            }
        })
        .map_err(|e| wasmtime::Error::msg(format!("GpuThread: {e}")))?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(
            rx.recv_timeout(Duration::from_secs(1))
                .map_err(|e| wasmtime::Error::msg(format!("GpuThread: {e}")))?,
        );
    }
    Ok(out)
}

fn register(
    linker: &mut Linker<TestHost>,
    configured: Arc<AtomicBool>,
    textures: Arc<AtomicU32>,
    submits: Arc<AtomicU32>,
    presents: Arc<AtomicU32>,
) -> wasmtime::Result<()> {
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
        "gpu-texture",
        ResourceType::host::<GpuTexture>(),
        |mut store, rep| {
            let resource = Resource::<GpuTexture>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
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
        "gpu-command-buffer",
        ResourceType::host::<GpuCommandBuffer>(),
        |mut store, rep| {
            let resource = Resource::<GpuCommandBuffer>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.func_wrap("get-device", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuDevice)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-queue", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuQueue)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap("get-command-buffer", |mut store, ()| {
        let resource = store.data_mut().table.push(GpuCommandBuffer)?;
        Ok((resource,))
    })?;
    let submits_cb = submits.clone();
    webgpu.func_wrap(
        "[method]gpu-queue.submit",
        move |mut store,
              (queue, commands): (Resource<GpuQueue>, Vec<Resource<GpuCommandBuffer>>)| {
            store.data_mut().table.get(&queue)?;
            assert_eq!(commands.len(), 1, "guest submits one command buffer");
            store.data_mut().table.get(&commands[0])?;
            submits_cb.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    )?;

    let mut surface = linker.instance("wasi-gfx:surface/surface@0.2.0")?;
    surface.resource(
        "surface",
        ResourceType::host::<GfxSurface>(),
        |mut store, rep| {
            let resource = Resource::<GfxSurface>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    surface.func_wrap(
        "[constructor]surface",
        |mut store, (_desc,): (GfxSurfaceCreateDesc,)| {
            let resource = store.data_mut().table.push(GfxSurface)?;
            Ok((resource,))
        },
    )?;
    surface.func_wrap(
        "[method]surface.on-frame",
        |mut store, (this,): (Resource<GfxSurface>,)| {
            store.data_mut().table.get(&this)?;
            let evs = gfx_on_frame_events(2)?;
            let reader = StreamReader::new(&mut store, evs)?;
            Ok((reader,))
        },
    )?;

    let mut sw = linker.instance("wasi-gfx:surface/surface-webgpu@0.2.0")?;
    sw.resource(
        "context",
        ResourceType::host::<GfxWebGpuContext>(),
        |mut store, rep| {
            let resource = Resource::<GfxWebGpuContext>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    sw.func_wrap(
        "[constructor]context",
        |mut store, (surf,): (Resource<GfxSurface>,)| {
            store.data_mut().table.get(&surf)?;
            let resource = store.data_mut().table.push(GfxWebGpuContext)?;
            Ok((resource,))
        },
    )?;
    sw.func_wrap(
        "[method]context.configure",
        move |mut store, (ctx, config): (Resource<GfxWebGpuContext>, GpuCanvasConfiguration)| {
            store.data_mut().table.get(&ctx)?;
            store.data_mut().table.get(&config.device)?;
            configured.store(true, Ordering::SeqCst);
            Ok(())
        },
    )?;
    let textures_cb = textures.clone();
    sw.func_wrap(
        "[method]context.get-current-texture",
        move |mut store, (ctx,): (Resource<GfxWebGpuContext>,)| {
            store.data_mut().table.get(&ctx)?;
            textures_cb.fetch_add(1, Ordering::SeqCst);
            let resource = store.data_mut().table.push(GpuTexture)?;
            Ok((resource,))
        },
    )?;
    sw.func_wrap(
        "[method]context.present",
        move |mut store, (ctx,): (Resource<GfxWebGpuContext>,)| {
            store.data_mut().table.get(&ctx)?;
            presents.fetch_add(1, Ordering::SeqCst);
            Ok(())
        },
    )?;
    Ok(())
}

#[test]
fn wasi_gfx_frame_loop_two_presents() -> wasmtime::Result<()> {
    WROTE_ON_GPU_THREAD.store(false, Ordering::SeqCst);
    let configured = Arc::new(AtomicBool::new(false));
    let textures = Arc::new(AtomicU32::new(0));
    let submits = Arc::new(AtomicU32::new(0));
    let presents = Arc::new(AtomicU32::new(0));
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/wasi/gfx_frame_loop.wasm"
    ))?;
    let component = Component::new(&engine, bytes)?;
    let mut linker = Linker::new(&engine);
    register(
        &mut linker,
        configured.clone(),
        textures.clone(),
        submits.clone(),
        presents.clone(),
    )?;
    let mut store = Store::new(
        &engine,
        TestHost {
            table: ResourceTable::new(),
        },
    );
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
    assert_eq!(v, 2, "guest must loop two on-frame events");
    assert!(configured.load(Ordering::SeqCst), "context.configure");
    assert_eq!(textures.load(Ordering::SeqCst), 2, "get-current-texture ×2");
    assert_eq!(submits.load(Ordering::SeqCst), 2, "queue.submit ×2");
    assert_eq!(presents.load(Ordering::SeqCst), 2, "context.present ×2");
    assert!(
        WROTE_ON_GPU_THREAD.load(Ordering::SeqCst),
        "frame-events must be produced on a thread named GpuThread"
    );
    Ok(())
}
