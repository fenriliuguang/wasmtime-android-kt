//! Store host state: Kotlin callbacks + u32-rep widget / gpu resources.

use crate::native_gpu::NativeGpuHost;
use jni::objects::GlobalRef;
use std::collections::HashMap;
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
    /// In-process Dawn C consume ([`NativeGpuHost`]). Unset → JNI default
    /// (`GpuBackend::JniBackend`).
    pub native_gpu: Option<NativeGpuHost>,
    /// P010-GFXV: 1-slot `on-frame` vsync gate (Choreographer → GpuThread write).
    pub gfx_on_frame: Arc<GfxOnFrameGate>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            table: ResourceTable::new(),
            add_cb: None,
            experimental_host_cb: None,
            native_gpu: None,
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
        let target = match g.last_take_gen {
            Some(prev) => prev.saturating_add(need),
            None => start_gen.saturating_add(need),
        };
        loop {
            if g.post_generation >= target {
                g.pending = false;
                g.in_frame = true;
                g.last_take_gen = Some(g.post_generation);
                Self::note_take_vsync(&mut g);
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
            g.last_take_wasi_ns = g
                .last_take_wasi_ns
                .saturating_add(g.last_post_ns.saturating_sub(g.last_take_vsync_ns));
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
