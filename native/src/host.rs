//! Store host state: Kotlin callbacks + u32-rep widget / gpu resources.

use crate::native_gpu::NativeGpuHost;
use jni::objects::GlobalRef;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Instant;
use wasmtime::component::ResourceTable;

static WASI_MONOTONIC_START: OnceLock<Instant> = OnceLock::new();

/// Process-elapsed ns for `wasi:clocks/monotonic-clock#now` (and H3 vsync seed).
pub fn wasi_monotonic_now_ns() -> u64 {
    WASI_MONOTONIC_START
        .get_or_init(Instant::now)
        .elapsed()
        .as_nanos() as u64
}

/// D3 skip-present probe. `0`/`1` = off (product 1:1). Device:
/// `adb shell setprop debug.wasmtime.gfx.skip_present_n 6` then restart the app.
fn hitch_skip_present_n() -> u32 {
    static N: OnceLock<u32> = OnceLock::new();
    *N.get_or_init(|| {
        if let Ok(s) = std::env::var("WASMTIME_GFX_SKIP_PRESENT_N") {
            if let Ok(n) = s.parse::<u32>() {
                return n;
            }
        }
        #[cfg(target_os = "android")]
        {
            let mut buf = [0u8; 92];
            unsafe extern "C" {
                fn __system_property_get(name: *const i8, value: *mut i8) -> i32;
            }
            let n = unsafe {
                __system_property_get(
                    c"debug.wasmtime.gfx.skip_present_n".as_ptr() as *const i8,
                    buf.as_mut_ptr() as *mut i8,
                )
            };
            if n > 0 {
                if let Ok(s) = std::str::from_utf8(&buf[..n as usize]) {
                    if let Ok(v) = s.parse::<u32>() {
                        return v;
                    }
                }
            }
        }
        0
    })
}

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
    /// Kotlin [ExperimentalHostCallbacks] for experimental CM host (M3/M4)
    /// and leftover [`crate::gpu_dispatch::GpuBackend::JniBackend`].
    pub experimental_host_cb: Option<GlobalRef>,
    /// In-process Dawn C consume ([`NativeGpuHost`]). Unset → JNI leftover
    /// (`GpuBackend::JniBackend`; unwired or `dawn-jni`).
    pub native_gpu: Option<NativeGpuHost>,
    /// Store window handle (`ANativeWindow*`) until NativeGpu is selected.
    pub canvas_native_window: i64,
    pub canvas_width: u32,
    pub canvas_height: u32,
    /// P010-GFXV: 1-slot `on-frame` vsync gate (Choreographer → GpuThread write).
    pub gfx_on_frame: Arc<GfxOnFrameGate>,
    /// GFX-SIZE: 1-slot `on-resize` (bound window / `request-set-size`).
    pub gfx_on_resize: Arc<GfxOnResizeGate>,
    /// Bounded `on-pointer-*` / `on-key-*` queues (Store post → guest stream).
    pub gfx_input: Arc<GfxInputHost>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            table: ResourceTable::new(),
            add_cb: None,
            experimental_host_cb: None,
            native_gpu: None,
            canvas_native_window: 0,
            canvas_width: 0,
            canvas_height: 0,
            gfx_on_frame: GfxOnFrameGate::new(),
            gfx_on_resize: GfxOnResizeGate::new(),
            gfx_input: GfxInputHost::new(),
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
    /// Guest has consumed a beat and not yet started the next wait.
    /// Choreographer posts in this window are dropped so the next
    /// `on-frame` read waits a *fresh* vsync (rAF phase lock). Taking a
    /// beat that arrived mid-frame caused present-present-gap hitching.
    in_frame: bool,
    closed: bool,
    dropped: u32,
    consumed: u32,
    /// Choreographer `frameTimeNanos` of the last `post` (0 = none).
    last_post_ns: u64,
    /// EWMA of post-to-post interval; 0 until the first sample.
    period_ns: u64,
    /// Monotonic count of `post` calls (including in-frame drops).
    post_generation: u64,
    /// `post_generation` at the last successful `wait_take` (`None` = never).
    last_take_gen: Option<u64>,
    /// Incremented at the start of each `wait_take` (tests wait until the
    /// waiter holds the condvar before posting the unblocking beat).
    wait_epoch: u64,
    /// Choreographer ns of the beat consumed by the last take (0 = none).
    last_take_vsync_ns: u64,
    /// WASI instant of that beat (vsync deltas after the first take).
    last_take_wasi_ns: u64,
    /// D3 probe: skip every Nth take (`0`/`1` = off). Guest waits the next
    /// vsync instead of presenting, so BLAST can drain. Not a product default.
    skip_present_n: u32,
}

pub enum GfxOnFrameTake {
    Item,
    Eof,
    Cancelled,
}

impl GfxOnFrameGate {
    pub fn new() -> Arc<Self> {
        let _ = wasi_monotonic_now_ns();
        Arc::new(Self {
            inner: Mutex::new(GfxOnFrameInner {
                pending: false,
                in_frame: false,
                closed: false,
                dropped: 0,
                consumed: 0,
                last_post_ns: 0,
                period_ns: 0,
                post_generation: 0,
                last_take_gen: None,
                wait_epoch: 0,
                last_take_vsync_ns: 0,
                last_take_wasi_ns: 0,
                skip_present_n: hitch_skip_present_n(),
            }),
            cv: Condvar::new(),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GfxOnFrameInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Fill the 1-slot. Drop if guest is still in a frame. Every post still
    /// advances `post_generation` so 120 Hz can count two beats since last take (H2).
    pub fn post(&self, frame_time_nanos: i64) {
        let mut g = self.lock();
        g.post_generation = g.post_generation.saturating_add(1);
        if frame_time_nanos > 0 {
            let ts = frame_time_nanos as u64;
            if g.last_post_ns > 0 {
                let dt = ts.saturating_sub(g.last_post_ns);
                if (2_000_000..25_000_000).contains(&dt) {
                    g.period_ns = if g.period_ns == 0 {
                        dt
                    } else {
                        (g.period_ns.saturating_mul(3).saturating_add(dt)) / 4
                    };
                }
            }
            g.last_post_ns = ts;
        }
        if g.closed {
            return;
        }
        if g.in_frame {
            g.dropped = g.dropped.saturating_add(1);
            return;
        }
        if g.pending {
            g.dropped = g.dropped.saturating_add(1);
            self.cv.notify_one();
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

    fn need_beats(_period_ns: u64) -> u64 {
        // 1:1 with Choreographer. Half-rate on a 120-capable VRR panel
        // (60/90/120 alternativeRates) let SurfaceFlinger rewind 3–4 BLAST
        // images every few seconds (H27).
        1
    }

    /// Block until a beat or close. Pin `on-frame` is a sync `func` and this
    /// repo does not enable stackful CM async, so `poll_produce` must not
    /// return `Pending` (guest WAT traps on stream.read BLOCKED).
    /// Phase lock: one post since the previous take. Latch `last_take_gen` to
    /// the current generation so a stall's queued vsyncs become **one** present
    /// (not a burst). Do **not** also wait `start_gen+1`: guest work that
    /// straddles one 8.3 ms vsync then forced 60 fps on a 120 Hz Fifo panel.
    pub fn wait_take(&self, finish: bool) -> GfxOnFrameTake {
        let mut g = self.lock();
        g.in_frame = false;
        g.pending = false;
        g.wait_epoch = g.wait_epoch.saturating_add(1);
        let need = Self::need_beats(g.period_ns);
        let start_gen = g.post_generation;
        let mut target = match g.last_take_gen {
            Some(prev) => prev.saturating_add(need),
            None => start_gen.saturating_add(need),
        };
        loop {
            if g.post_generation >= target {
                g.pending = false;
                g.last_take_gen = Some(g.post_generation);
                Self::note_take_vsync(&mut g);
                g.consumed = g.consumed.saturating_add(1);
                let n = g.skip_present_n;
                if n >= 2 && g.consumed % n == 0 {
                    // Consume this vsync without producing a BLAST image.
                    g.in_frame = false;
                    target = g.post_generation.saturating_add(need);
                    continue;
                }
                g.in_frame = true;
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
    /// Do not clear `in_frame` (that let mid-frame posts become pending).
    pub fn wait_ready(&self, finish: bool) -> GfxOnFrameTake {
        let mut g = self.lock();
        loop {
            if g.pending || g.in_frame {
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

    #[cfg(test)]
    fn wait_epoch(&self) -> u64 {
        self.lock().wait_epoch
    }

    fn note_take_vsync(g: &mut GfxOnFrameInner) {
        if g.last_post_ns == 0 {
            g.last_take_wasi_ns = 0;
            return;
        }
        if g.last_take_vsync_ns > 0 && g.last_post_ns >= g.last_take_vsync_ns {
            let dt = g.last_post_ns.saturating_sub(g.last_take_vsync_ns);
            g.last_take_wasi_ns = g.last_take_wasi_ns.saturating_add(dt);
        } else {
            g.last_take_wasi_ns = wasi_monotonic_now_ns().max(1);
        }
        g.last_take_vsync_ns = g.last_post_ns;
    }

    /// Host-only H3: while guest is in a frame, `clocks.now` is the vsync
    /// instant of the beat that released `on-frame` (not GpuThread wakeup).
    pub fn in_frame_instant_ns(&self, fallback: u64) -> u64 {
        let g = self.lock();
        if g.in_frame && g.last_take_wasi_ns > 0 {
            g.last_take_wasi_ns
        } else {
            fallback
        }
    }

    /// Choreographer `frameTimeNanos` of the beat consumed by the last take
    /// (`0` = none).
    pub fn last_take_vsync_ns(&self) -> u64 {
        self.lock().last_take_vsync_ns
    }
}

/// Latest bound-window size for `stream<resize-event>`. Coalesce; do not queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GfxResizeSize {
    pub width: u32,
    pub height: u32,
}

pub struct GfxOnResizeGate {
    inner: Mutex<GfxOnResizeInner>,
    cv: Condvar,
}

struct GfxOnResizeInner {
    pending: Option<GfxResizeSize>,
    closed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GfxOnResizeTake {
    Item(GfxResizeSize),
    Eof,
    Cancelled,
}

impl GfxOnResizeGate {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(GfxOnResizeInner {
                pending: None,
                closed: false,
            }),
            cv: Condvar::new(),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GfxOnResizeInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Keep the latest size. Pin `on-resize` is a sync `func`; `poll_produce`
    /// must not return `Pending`.
    pub fn post(&self, width: u32, height: u32) {
        let mut g = self.lock();
        if g.closed {
            return;
        }
        g.pending = Some(GfxResizeSize { width, height });
        self.cv.notify_one();
    }

    pub fn close(&self) {
        let mut g = self.lock();
        g.closed = true;
        self.cv.notify_all();
    }

    pub fn wait_take(&self, finish: bool) -> GfxOnResizeTake {
        let mut g = self.lock();
        loop {
            if let Some(ev) = g.pending.take() {
                return GfxOnResizeTake::Item(ev);
            }
            if g.closed {
                return GfxOnResizeTake::Eof;
            }
            if finish {
                return GfxOnResizeTake::Cancelled;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub fn wait_ready(&self, finish: bool) -> GfxOnResizeTake {
        let mut g = self.lock();
        loop {
            if let Some(ev) = g.pending {
                return GfxOnResizeTake::Item(ev);
            }
            if g.closed {
                return GfxOnResizeTake::Eof;
            }
            if finish {
                return GfxOnResizeTake::Cancelled;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }
}

/// Bounded input queue for pin `on-pointer-*` / `on-key-*`. Clicks are not
/// coalesced (unlike resize). Full queue drops the oldest sample.
const GFX_INPUT_CAP: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GfxPointerSample {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GfxKeySample {
    /// WIT `key` discriminant, or `None` when the Android code is unmapped.
    pub key: Option<u8>,
    pub text: Option<String>,
    pub alt_key: bool,
    pub ctrl_key: bool,
    pub meta_key: bool,
    pub shift_key: bool,
}

#[derive(Debug)]
pub enum GfxInputTake<T> {
    Item(T),
    Eof,
    Cancelled,
}

pub struct GfxInputGate<T> {
    inner: Mutex<GfxInputInner<T>>,
    cv: Condvar,
}

struct GfxInputInner<T> {
    queue: VecDeque<T>,
    closed: bool,
}

pub type GfxPointerGate = GfxInputGate<GfxPointerSample>;
pub type GfxKeyGate = GfxInputGate<GfxKeySample>;

impl<T: Clone> GfxInputGate<T> {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(GfxInputInner {
                queue: VecDeque::new(),
                closed: false,
            }),
            cv: Condvar::new(),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, GfxInputInner<T>> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn post(&self, item: T) {
        let mut g = self.lock();
        if g.closed {
            return;
        }
        if g.queue.len() >= GFX_INPUT_CAP {
            g.queue.pop_front();
        }
        g.queue.push_back(item);
        self.cv.notify_one();
    }

    pub fn close(&self) {
        let mut g = self.lock();
        g.closed = true;
        self.cv.notify_all();
    }

    /// Pin input streams are sync `func`s; `poll_produce` must not return
    /// `Pending` (guest WAT traps on stream.read BLOCKED).
    pub fn wait_take(&self, finish: bool) -> GfxInputTake<T> {
        let mut g = self.lock();
        loop {
            if let Some(ev) = g.queue.pop_front() {
                return GfxInputTake::Item(ev);
            }
            if g.closed {
                return GfxInputTake::Eof;
            }
            if finish {
                return GfxInputTake::Cancelled;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub fn wait_ready(&self, finish: bool) -> GfxInputTake<T> {
        let mut g = self.lock();
        loop {
            if let Some(ev) = g.queue.front() {
                return GfxInputTake::Item(ev.clone());
            }
            if g.closed {
                return GfxInputTake::Eof;
            }
            if finish {
                return GfxInputTake::Cancelled;
            }
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }

    pub fn len(&self) -> usize {
        self.lock().queue.len()
    }
}

/// Five pin input streams. UI-thread post uses the process map, not Store.
pub struct GfxInputHost {
    pub pointer_up: Arc<GfxPointerGate>,
    pub pointer_down: Arc<GfxPointerGate>,
    pub pointer_move: Arc<GfxPointerGate>,
    pub key_up: Arc<GfxKeyGate>,
    pub key_down: Arc<GfxKeyGate>,
}

impl GfxInputHost {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pointer_up: GfxInputGate::new(),
            pointer_down: GfxInputGate::new(),
            pointer_move: GfxInputGate::new(),
            key_up: GfxInputGate::new(),
            key_down: GfxInputGate::new(),
        })
    }

    pub fn close(&self) {
        self.pointer_up.close();
        self.pointer_down.close();
        self.pointer_move.close();
        self.key_up.close();
        self.key_down.close();
    }

    pub fn post_pointer(&self, kind: i32, x: f64, y: f64) {
        let ev = GfxPointerSample { x, y };
        match kind {
            0 => self.pointer_up.post(ev),
            1 => self.pointer_down.post(ev),
            2 => self.pointer_move.post(ev),
            _ => {}
        }
    }

    pub fn post_key(&self, down: bool, sample: GfxKeySample) {
        if down {
            self.key_down.post(sample);
        } else {
            self.key_up.post(sample);
        }
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

static GFX_INPUT: OnceLock<Mutex<HashMap<i64, Arc<GfxInputHost>>>> = OnceLock::new();

fn gfx_input_map() -> &'static Mutex<HashMap<i64, Arc<GfxInputHost>>> {
    GFX_INPUT.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn gfx_input_register(handle: i64, gates: Arc<GfxInputHost>) {
    gfx_input_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(handle, gates);
}

pub fn gfx_input_lookup(handle: i64) -> Option<Arc<GfxInputHost>> {
    gfx_input_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&handle)
        .cloned()
}

pub fn gfx_input_unregister(handle: i64) -> Option<Arc<GfxInputHost>> {
    gfx_input_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&handle)
}

#[cfg(test)]
mod gfx_h3_instant {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn in_frame_instant_follows_vsync_dt() {
        let gate = GfxOnFrameGate::new();
        let waiter = gate.clone();
        let first = thread::spawn(move || waiter.wait_take(false));
        thread::sleep(Duration::from_millis(20));
        gate.post(8_333_333);
        gate.post(16_666_666);
        first.join().expect("first take");
        let a = gate.in_frame_instant_ns(0);
        assert!(a > 0, "first take seeds WASI instant");

        let second = spawn_blocked_wait(&gate);
        gate.post(25_000_000);
        gate.post(33_333_333);
        second.join().expect("second take");
        let b = gate.in_frame_instant_ns(0);
        assert_eq!(b.saturating_sub(a), 16_666_667);
    }

    #[test]
    fn stall_does_not_drain_queued_vsyncs() {
        let gate = GfxOnFrameGate::new();
        let waiter = gate.clone();
        let first = thread::spawn(move || waiter.wait_take(false));
        thread::sleep(Duration::from_millis(20));
        gate.post(8_333_333);
        gate.post(16_666_666);
        first.join().expect("first take");

        for i in 1..=4i64 {
            gate.post(16_666_666 + i * 8_333_333);
        }
        // One take eats the backlog (latch to current gen).
        assert!(matches!(gate.wait_take(false), GfxOnFrameTake::Item));
        // The next take must wait for a fresh vsync, not drain leftovers.
        let third = spawn_blocked_wait(&gate);
        assert!(!third.is_finished());
        gate.post(16_666_666 + 5 * 8_333_333);
        third.join().expect("third take");
    }

    fn spawn_blocked_wait(gate: &Arc<GfxOnFrameGate>) -> thread::JoinHandle<GfxOnFrameTake> {
        let epoch = gate.wait_epoch();
        let waiter = gate.clone();
        let handle = thread::spawn(move || waiter.wait_take(false));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if gate.wait_epoch() != epoch {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "wait_take thread did not start"
            );
            assert!(
                !handle.is_finished(),
                "wait_take returned before a fresh vsync"
            );
            thread::sleep(Duration::from_millis(1));
        }
        assert!(
            !handle.is_finished(),
            "wait_take returned before a fresh vsync"
        );
        handle
    }
}

#[cfg(test)]
mod gfx_input_queue {
    use super::*;

    #[test]
    fn pointer_queue_preserves_order() {
        let gate = GfxPointerGate::new();
        gate.post(GfxPointerSample { x: 1.0, y: 2.0 });
        gate.post(GfxPointerSample { x: 3.0, y: 4.0 });
        match gate.wait_take(true) {
            GfxInputTake::Item(a) => {
                assert_eq!(a.x, 1.0);
                assert_eq!(a.y, 2.0);
            }
            other => panic!("expected first sample, got {other:?}"),
        }
        match gate.wait_take(true) {
            GfxInputTake::Item(b) => {
                assert_eq!(b.x, 3.0);
                assert_eq!(b.y, 4.0);
            }
            other => panic!("expected second sample, got {other:?}"),
        }
    }

    #[test]
    fn pointer_cap_drops_oldest() {
        let gate = GfxPointerGate::new();
        for i in 0..=GFX_INPUT_CAP {
            gate.post(GfxPointerSample {
                x: i as f64,
                y: 0.0,
            });
        }
        assert_eq!(gate.len(), GFX_INPUT_CAP);
        match gate.wait_take(true) {
            GfxInputTake::Item(a) => assert_eq!(a.x, 1.0),
            other => panic!("expected oldest-dropped sample, got {other:?}"),
        }
    }

    #[test]
    fn close_yields_eof() {
        let gate = GfxPointerGate::new();
        gate.close();
        assert!(matches!(gate.wait_take(false), GfxInputTake::Eof));
    }

    #[test]
    fn key_post_reaches_down_gate() {
        let host = GfxInputHost::new();
        host.post_key(
            true,
            GfxKeySample {
                key: Some(19), // key-a is the 20th variant (0-based 19)
                text: Some("A".into()),
                alt_key: false,
                ctrl_key: false,
                meta_key: false,
                shift_key: true,
            },
        );
        match host.key_down.wait_take(true) {
            GfxInputTake::Item(ev) => {
                assert_eq!(ev.key, Some(19));
                assert_eq!(ev.text.as_deref(), Some("A"));
                assert!(ev.shift_key);
            }
            other => panic!("expected key-down, got {other:?}"),
        }
        assert!(matches!(
            host.key_up.wait_take(true),
            GfxInputTake::Cancelled
        ));
    }
}
