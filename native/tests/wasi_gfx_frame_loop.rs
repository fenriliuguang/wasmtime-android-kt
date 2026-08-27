//! P010-GFXV: product guest `get-gpu` → `request-adapter` → `request-device`
//! then vsync-paced `on-frame` loop → get-current-texture → submit → present.
//! Helper thread named `GpuThread` posts 1-slot beats (drop unconsumed); close
//! the stream so `run` unblocks. No JS-style callback. Not two events at construct.

use futures::channel::oneshot;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

use wasmtime::component::{
    flags, Component, ComponentType, Destination, Lift, Linker, Lower, Resource, ResourceTable,
    ResourceType, StreamProducer, StreamReader, StreamResult,
};
use wasmtime::{Config, Engine, Store, StoreContextMut};

#[path = "wasi_webgpu_method/texture_format.rs"]
mod texture_format;
use texture_format::GpuTextureFormat;

static WROTE_ON_GPU_THREAD: AtomicBool = AtomicBool::new(false);
static PENDING_AT_ON_FRAME: AtomicBool = AtomicBool::new(true);
static DROPPED_BEATS: AtomicU32 = AtomicU32::new(0);

struct GfxSurface;
struct GfxWebGpuContext;
struct Gpu;
struct GpuAdapter;
struct GpuDevice;
struct GpuTexture;
struct GpuQueue;
struct GpuCommandBuffer;
struct RecordOptionGpuSize64;

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuPowerPreference {
    #[component(name = "low-power")]
    LowPower,
    #[component(name = "high-performance")]
    HighPerformance,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
struct GpuRequestAdapterOptions {
    #[component(name = "feature-level")]
    feature_level: Option<String>,
    #[component(name = "power-preference")]
    power_preference: Option<GpuPowerPreference>,
    #[component(name = "force-fallback-adapter")]
    force_fallback_adapter: Option<bool>,
    #[component(name = "xr-compatible")]
    xr_compatible: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ComponentType, Lift, Lower)]
#[component(enum)]
#[repr(u8)]
#[allow(dead_code)]
enum GpuFeatureName {
    #[component(name = "core-features-and-limits")]
    CoreFeaturesAndLimits,
    #[component(name = "depth-clip-control")]
    DepthClipControl,
    #[component(name = "depth32float-stencil8")]
    Depth32floatStencil8,
    #[component(name = "texture-compression-bc")]
    TextureCompressionBc,
    #[component(name = "texture-compression-bc-sliced3d")]
    TextureCompressionBcSliced3d,
    #[component(name = "texture-compression-etc2")]
    TextureCompressionEtc2,
    #[component(name = "texture-compression-astc")]
    TextureCompressionAstc,
    #[component(name = "texture-compression-astc-sliced3d")]
    TextureCompressionAstcSliced3d,
    #[component(name = "timestamp-query")]
    TimestampQuery,
    #[component(name = "indirect-first-instance")]
    IndirectFirstInstance,
    #[component(name = "shader-f16")]
    ShaderF16,
    #[component(name = "rg11b10ufloat-renderable")]
    Rg11b10ufloatRenderable,
    #[component(name = "bgra8unorm-storage")]
    Bgra8unormStorage,
    #[component(name = "float32-filterable")]
    Float32Filterable,
    #[component(name = "float32-blendable")]
    Float32Blendable,
    #[component(name = "clip-distances")]
    ClipDistances,
    #[component(name = "dual-source-blending")]
    DualSourceBlending,
    #[component(name = "subgroups")]
    Subgroups,
    #[component(name = "texture-formats-tier1")]
    TextureFormatsTier1,
    #[component(name = "texture-formats-tier2")]
    TextureFormatsTier2,
    #[component(name = "primitive-index")]
    PrimitiveIndex,
    #[component(name = "texture-component-swizzle")]
    TextureComponentSwizzle,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuQueueDescriptor {
    label: Option<String>,
}

#[derive(Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct GpuDeviceDescriptor {
    #[component(name = "required-features")]
    required_features: Option<Vec<GpuFeatureName>>,
    #[component(name = "required-limits")]
    required_limits: Option<Resource<RecordOptionGpuSize64>>,
    #[component(name = "default-queue")]
    default_queue: Option<GpuQueueDescriptor>,
    label: Option<String>,
}

#[derive(Clone, Copy, Debug, ComponentType, Lift, Lower)]
#[component(variant)]
#[allow(dead_code)]
enum RequestDeviceErrorKind {
    #[component(name = "type-error")]
    TypeError,
    #[component(name = "operation-error")]
    OperationError,
}

#[derive(Clone, Debug, ComponentType, Lift, Lower)]
#[component(record)]
#[allow(dead_code)]
struct RequestDeviceError {
    kind: RequestDeviceErrorKind,
    message: String,
}

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
    gate: Arc<GfxOnFrameGate>,
}

struct GfxOnFrameGate {
    inner: Mutex<GfxOnFrameInner>,
    cv: Condvar,
}

struct GfxOnFrameInner {
    pending: bool,
    in_frame: bool,
    closed: bool,
    dropped: u32,
    consumed: u32,
}

enum GfxOnFrameTake {
    Item,
    Eof,
    Cancelled,
}

impl GfxOnFrameGate {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(GfxOnFrameInner {
                pending: false,
                in_frame: false,
                closed: false,
                dropped: 0,
                consumed: 0,
            }),
            cv: Condvar::new(),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GfxOnFrameInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn post(&self) {
        let mut g = self.lock();
        if g.closed {
            return;
        }
        if g.pending || g.in_frame {
            g.dropped = g.dropped.saturating_add(1);
            DROPPED_BEATS.store(g.dropped, Ordering::SeqCst);
            return;
        }
        g.pending = true;
        self.cv.notify_one();
    }

    fn close(&self) {
        let mut g = self.lock();
        g.closed = true;
        self.cv.notify_all();
    }

    fn wait_take(&self, finish: bool) -> GfxOnFrameTake {
        let mut g = self.lock();
        g.in_frame = false;
        loop {
            if g.pending {
                g.pending = false;
                g.in_frame = true;
                g.consumed = g.consumed.saturating_add(1);
                return GfxOnFrameTake::Item;
            }
            if g.closed {
                return GfxOnFrameTake::Eof;
            }
            if finish {
                return GfxOnFrameTake::Cancelled;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    fn wait_ready(&self, finish: bool) -> GfxOnFrameTake {
        let mut g = self.lock();
        g.in_frame = false;
        loop {
            if g.pending {
                return GfxOnFrameTake::Item;
            }
            if g.closed {
                return GfxOnFrameTake::Eof;
            }
            if finish {
                return GfxOnFrameTake::Cancelled;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    fn has_pending(&self) -> bool {
        self.lock().pending
    }

    fn is_closed(&self) -> bool {
        self.lock().closed
    }

    fn consumed(&self) -> u32 {
        self.lock().consumed
    }
}

struct GfxOnFrameProducer {
    gate: Arc<GfxOnFrameGate>,
}

impl StreamProducer<TestHost> for GfxOnFrameProducer {
    type Item = GfxFrameEvent;
    type Buffer = Option<GfxFrameEvent>;

    fn poll_produce<'a>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut store: StoreContextMut<'a, TestHost>,
        mut destination: Destination<'a, Self::Item, Self::Buffer>,
        finish: bool,
    ) -> Poll<wasmtime::Result<StreamResult>> {
        let _ = cx;
        if destination.remaining(&mut store) == Some(0) {
            return Poll::Ready(Ok(match self.gate.wait_ready(finish) {
                GfxOnFrameTake::Item => StreamResult::Completed,
                GfxOnFrameTake::Eof => StreamResult::Dropped,
                GfxOnFrameTake::Cancelled => StreamResult::Cancelled,
            }));
        }
        match self.gate.wait_take(finish) {
            GfxOnFrameTake::Item => {
                destination.set_buffer(Some(GfxFrameEvent { nothing: true }));
                if self.gate.is_closed() {
                    Poll::Ready(Ok(StreamResult::Dropped))
                } else {
                    Poll::Ready(Ok(StreamResult::Completed))
                }
            }
            GfxOnFrameTake::Eof => Poll::Ready(Ok(StreamResult::Dropped)),
            GfxOnFrameTake::Cancelled => Poll::Ready(Ok(StreamResult::Cancelled)),
        }
    }
}

fn start_vsync(gate: Arc<GfxOnFrameGate>) -> wasmtime::Result<()> {
    thread::Builder::new()
        .name("GpuThread".into())
        .spawn(move || {
            WROTE_ON_GPU_THREAD.store(
                thread::current().name() == Some("GpuThread"),
                Ordering::SeqCst,
            );
            gate.post();
            gate.post();
            let deadline = Instant::now() + Duration::from_secs(5);
            while gate.consumed() < 2 && Instant::now() < deadline {
                gate.post();
                thread::sleep(Duration::from_millis(2));
            }
            gate.close();
        })
        .map_err(|e| wasmtime::Error::msg(format!("GpuThread: {e}")))?;
    Ok(())
}

fn register(
    linker: &mut Linker<TestHost>,
    configured: Arc<AtomicBool>,
    textures: Arc<AtomicU32>,
    submits: Arc<AtomicU32>,
    presents: Arc<AtomicU32>,
) -> wasmtime::Result<()> {
    let mut webgpu = linker.instance("wasi:webgpu/webgpu@0.3.0-rc.2")?;
    webgpu.resource("gpu", ResourceType::host::<Gpu>(), |mut store, rep| {
        let resource = Resource::<Gpu>::new_own(rep);
        store.data_mut().table.delete(resource)?;
        Ok(())
    })?;
    webgpu.resource(
        "gpu-adapter",
        ResourceType::host::<GpuAdapter>(),
        |mut store, rep| {
            let resource = Resource::<GpuAdapter>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
    webgpu.resource(
        "record-option-gpu-size64",
        ResourceType::host::<RecordOptionGpuSize64>(),
        |mut store, rep| {
            let resource = Resource::<RecordOptionGpuSize64>::new_own(rep);
            store.data_mut().table.delete(resource)?;
            Ok(())
        },
    )?;
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
    webgpu.func_wrap("get-gpu", |mut store, ()| {
        let resource = store.data_mut().table.push(Gpu)?;
        Ok((resource,))
    })?;
    webgpu.func_wrap_concurrent(
        "[method]gpu.request-adapter",
        |accessor, (gpu, _options): (Resource<Gpu>, Option<GpuRequestAdapterOptions>)| {
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&gpu).map(|_| ()))?;
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                let resource =
                    accessor.with(|mut access| access.data_mut().table.push(GpuAdapter))?;
                Ok((Some(resource),))
            })
        },
    )?;
    webgpu.func_wrap_concurrent(
        "[method]gpu-adapter.request-device",
        |accessor, (adapter, _descriptor): (Resource<GpuAdapter>, Option<GpuDeviceDescriptor>)| {
            Box::pin(async move {
                accessor.with(|mut access| access.data_mut().table.get(&adapter).map(|_| ()))?;
                let (tx, rx) = oneshot::channel::<()>();
                std::thread::spawn(move || {
                    let _ = tx.send(());
                });
                let _ = rx.await;
                let resource =
                    accessor.with(|mut access| access.data_mut().table.push(GpuDevice))?;
                Ok((Ok::<_, RequestDeviceError>(resource),))
            })
        },
    )?;
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
            let gate = store.data().gate.clone();
            PENDING_AT_ON_FRAME.store(gate.has_pending(), Ordering::SeqCst);
            start_vsync(gate.clone())?;
            let reader = StreamReader::new(&mut store, GfxOnFrameProducer { gate })?;
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
fn wasi_gfx_frame_loop_vsync_paced() -> wasmtime::Result<()> {
    WROTE_ON_GPU_THREAD.store(false, Ordering::SeqCst);
    PENDING_AT_ON_FRAME.store(true, Ordering::SeqCst);
    DROPPED_BEATS.store(0, Ordering::SeqCst);
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
            gate: GfxOnFrameGate::new(),
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
    assert!(
        v >= 2,
        "guest must loop at least two vsync-paced on-frame events, got {v}"
    );
    assert!(configured.load(Ordering::SeqCst), "context.configure");
    assert!(
        textures.load(Ordering::SeqCst) >= 2,
        "get-current-texture ×≥2"
    );
    assert!(submits.load(Ordering::SeqCst) >= 2, "queue.submit ×≥2");
    assert!(presents.load(Ordering::SeqCst) >= 2, "context.present ×≥2");
    assert!(
        WROTE_ON_GPU_THREAD.load(Ordering::SeqCst),
        "frame-events must be posted on a thread named GpuThread"
    );
    assert!(
        !PENDING_AT_ON_FRAME.load(Ordering::SeqCst),
        "on-frame must not pre-buffer events at construct"
    );
    assert!(
        DROPPED_BEATS.load(Ordering::SeqCst) >= 1,
        "unconsumed vsync beats must be dropped"
    );
    Ok(())
}
