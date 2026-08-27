//! Store host state: Kotlin callbacks + u32-rep widget / gpu resources.

use jni::objects::GlobalRef;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use wasmtime::component::ResourceTable;

#[derive(Debug)]
pub struct Widget {
    pub rep: u32,
}

/// Host representation of WIT `resource gpu` (W3 `[method]` slice). No L2 handle.
#[derive(Debug)]
pub struct Gpu;

/// Host representation of WIT `resource gpu-adapter`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuAdapter {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-device`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// S1 `get-device` still pushes `{ rep: 0 }`; L2 handles for that path are
/// obtained inside later methods. S3 `request-device` stores the real L2 rep.
#[derive(Debug)]
pub struct GpuDevice {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-command-buffer`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-command-buffer` still pushes `{ rep: 0 }`; S7 `encoder.finish` stores
/// the real L2 rep.
#[derive(Debug)]
pub struct GpuCommandBuffer {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-command-encoder`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-encoder` still pushes `{ rep: 0 }`; S6 `create-command-encoder` stores
/// the real L2 rep.
#[derive(Debug)]
pub struct GpuCommandEncoder {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-render-pass-encoder`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-pass` still pushes `{ rep: 0 }`; S6+ `begin-render-pass` stores the real L2 rep.
#[derive(Debug)]
pub struct GpuRenderPassEncoder {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-compute-pass-encoder`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-compute-pass` still pushes `{ rep: 0 }`; S8 `begin-compute-pass` stores
/// the real L2 rep.
#[derive(Debug)]
pub struct GpuComputePassEncoder {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-queue`.
/// `rep` is the Dawn / Cpu L2 handle (u32); Guest sees `own`/`borrow`, not this value.
/// Read by later slices (submit / write-*); S1 only stores it.
#[derive(Debug)]
pub struct GpuQueue {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-texture` (W3+ `[method]`).
/// `get-texture` still pushes `{ rep: 0 }`; S6+ `create-texture` stores the real L2 rep.
#[derive(Debug)]
pub struct GpuTexture {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-sampler`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuSampler {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-shader-module`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuShaderModule {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-bind-group-layout`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-bind-group-layout` still pushes `{ rep: 0 }`; S6+ `create-bind-group-layout`
/// stores the real L2 rep.
#[derive(Debug)]
pub struct GpuBindGroupLayout {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-pipeline-layout`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuPipelineLayout {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-bind-group`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuBindGroup {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-compute-pipeline`.
/// `get-compute-pipeline` still pushes `{ rep: 0 }`; S6+ `create-compute-pipeline`
/// stores the real L2 rep.
#[derive(Debug)]
pub struct GpuComputePipeline {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-render-pipeline`.
/// `get-render-pipeline` still pushes `{ rep: 0 }`; S6+ `create-render-pipeline`
/// stores the real L2 rep.
#[derive(Debug)]
pub struct GpuRenderPipeline {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-query-set`.
/// `get-query-set` still pushes `{ rep: 0 }`; L2 stores the real handle.
#[derive(Debug)]
pub struct GpuQuerySet {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-texture-view`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
#[derive(Debug)]
pub struct GpuTextureView {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-render-bundle`.
/// `get-render-bundle` still pushes `{ rep: 0 }`; S6+ `bundle-encoder.finish` stores it.
#[derive(Debug)]
pub struct GpuRenderBundle {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-render-bundle-encoder`.
/// `get-render-bundle-encoder` still pushes `{ rep: 0 }`.
#[derive(Debug)]
pub struct GpuRenderBundleEncoder {
    #[allow(dead_code)]
    pub rep: u32,
}

/// Host representation of WIT `resource gpu-buffer`.
/// `rep` is the Dawn / Cpu L2 handle; Guest sees `own`/`borrow`.
/// `get-buffer` still pushes `{ rep: 0 }`; S4 `create-buffer` stores the real L2 rep.
#[derive(Debug)]
pub struct GpuBuffer {
    #[allow(dead_code)]
    pub rep: u32,
}

pub struct HostState {
    pub table: ResourceTable,
    pub add_cb: Option<GlobalRef>,
    /// Kotlin [ExperimentalHostCallbacks] for experimental CM host (M3/M4).
    pub experimental_host_cb: Option<GlobalRef>,
    /// P010-GFXV: 1-slot `on-frame` vsync gate (Choreographer → GpuThread write).
    pub gfx_on_frame: Arc<GfxOnFrameGate>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            table: ResourceTable::new(),
            add_cb: None,
            experimental_host_cb: None,
            gfx_on_frame: GfxOnFrameGate::new(),
        }
    }
}

/// 1-slot vsync source for `stream<frame-event>`. Unconsumed beats are dropped.
pub struct GfxOnFrameGate {
    inner: Mutex<GfxOnFrameInner>,
    cv: Condvar,
}

struct GfxOnFrameInner {
    pending: bool,
    closed: bool,
    dropped: u32,
    consumed: u32,
}

pub enum GfxOnFrameTake {
    Item,
    Eof,
    Cancelled,
}

impl GfxOnFrameGate {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(GfxOnFrameInner {
                pending: false,
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

    /// Fill the 1-slot. Drop this beat if the previous event is unconsumed.
    pub fn post(&self) {
        let mut g = self.lock();
        if g.closed {
            return;
        }
        if g.pending {
            g.dropped = g.dropped.saturating_add(1);
            return;
        }
        g.pending = true;
        self.cv.notify_one();
    }

    pub fn close(&self) {
        let mut g = self.lock();
        g.closed = true;
        self.cv.notify_all();
    }

    /// Block until a beat or close. Pin `on-frame` is a sync `func` and this
    /// repo does not enable stackful CM async, so `poll_produce` must not
    /// return `Pending` (guest WAT traps on stream.read BLOCKED).
    pub fn wait_take(&self, finish: bool) -> GfxOnFrameTake {
        let mut g = self.lock();
        loop {
            if g.pending {
                g.pending = false;
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

    /// Zero-length readiness: wait without consuming the slot.
    pub fn wait_ready(&self, finish: bool) -> GfxOnFrameTake {
        let mut g = self.lock();
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

    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }
}

static GFX_GATES: OnceLock<Mutex<HashMap<i64, Arc<GfxOnFrameGate>>>> = OnceLock::new();

fn gfx_gates() -> &'static Mutex<HashMap<i64, Arc<GfxOnFrameGate>>> {
    GFX_GATES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn gfx_on_frame_register(handle: i64, gate: Arc<GfxOnFrameGate>) {
    gfx_gates()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(handle, gate);
}

pub fn gfx_on_frame_lookup(handle: i64) -> Option<Arc<GfxOnFrameGate>> {
    gfx_gates()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&handle)
        .cloned()
}

pub fn gfx_on_frame_unregister(handle: i64) -> Option<Arc<GfxOnFrameGate>> {
    gfx_gates()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&handle)
}
