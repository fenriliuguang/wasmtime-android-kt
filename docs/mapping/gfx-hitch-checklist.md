# Cube present hitch checklist

**English** | [中文](gfx-hitch-checklist.zh.md)

Not a cut queue. Living **device** investigation for the out-of-tree rotating cube (Vivo V2458A, Android 16, `arm64-v8a`, Mali-G925-Immortalis MC12, 120 Hz). Continuous `wasi-gfx` `on-frame`, not the 500 ms GFXV instrument. Do not vendor the demo. Do not file upstream GitHub issues.

Hitch **still observed** after: swapchain recycle + GPU fence, no 60 Hz cap, guest `monotonic-clock.now` delta, and dropping Choreographer beats while guest is in a frame. This page lists **every remaining cause** we can name, with a code check. Update the **Check** column when a row is closed or a fix lands. Draft PR stays open until the cube is visually smooth.

Related: [`gap-webgpu-wit-androidx.md`](gap-webgpu-wit-androidx.md) §5, [`threading-android.md`](threading-android.md) §8, [`frame-loop-suggestion.md`](frame-loop-suggestion.md). Guest clocks/dt lives in the **out-of-tree** examples repo.

**Check**

| Tag | Meaning |
|-----|---------|
| **Open** | Still a plausible hitch; not disproved on device |
| **Likely** | Code path runs every frame; strongest remaining suspects |
| **Mitigated** | Changed in this branch; hitch remains so not sufficient alone |
| **Closed** | Ruled out for *this* hitch (may still be a crash/leak fact) |
| **Guest** | Out-of-tree wasm; not this repo’s tree |
| **Trace** | Needs systrace / per-frame logs on device; not done |

## 1. Closed or mitigated (still listed so we do not rediscover)

| ID | Hypothesis | Check | Evidence |
|----|------------|-------|----------|
| C1 | BLAST / HandleTable leak of swapchain `GPUTexture` | **Closed** (crash) | Never `tryDrop` after present → hitching rose then GpuThread SIGSEGV `0x20` ~10 s. Recycle + GPU fence; crash gone on V2458A (~minutes). |
| C2 | `close()` same present / next acquire / CPU-frame ring | **Closed** (UAF) | Immediate close → `0x20` / `0x1f8`. CPU keep-last-N without fence → crash ~45 s. Keep last 3 **and** GPU done. |
| C3 | Host ~60 Hz cap on `postGfxVsync` | **Mitigated** | `MIN_GFX_VSYNC_NS` removed. Fast launch gone; hitch remains. Cap itself caused every-other-beat jitter on 120 Hz. |
| C4 | Guest `angle += const` per `on-frame` | **Mitigated** / **Guest** | Cube now uses `wasi:clocks/monotonic-clock#now` (rAF-style dt, 50 ms clamp). Pin `frame-event` is still `{ nothing: bool }`. |
| C5 | Consume a vsync that arrived mid-frame | **Mitigated** | Native `GfxOnFrameGate`: `post` drops if `pending \|\| in_frame`. Hitch remains → not the only cause. Native test `wasi_gfx_frame_loop_vsync_paced` updated. |
| C6 | GFXV instrument | **Closed** | `CLOSE_AFTER_VSYNC_MS = 500`; never saw seconds of present. Cpu recycle: `WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest`. |

## 2. Remaining causes (check in this order)

| ID | Hypothesis | Check | How we checked | Next if still Open |
|----|------------|-------|----------------|--------------------|
| H1 | After vsync, `get-current-texture` **waits previous GPU** (`awaitCanvasGpuDone`) so present is late vs scanout | **Likely** | `DawnWasiWebGpuHost.canvasContextGetCurrentTexture` awaits `lastCanvasSubmitDone` **before** acquire. Guest samples `now` *before* that wait, then presents late. Phase vs Mali/BLAST scanout varies with GPU time. | Move fence wait **after** present / off the vsync-to-present path; keep ring+fence for UAF. Device: log vsync→present ns. |
| H2 | `in_frame` drop + 120 Hz: work time straddles 8.3 ms → **60/120 Hz oscillation** (classic judder) | **Likely** | If one frame is 7 ms the next vsync is taken (120 Hz); if 10 ms that beat was dropped and we wait ~16.6 ms (60 Hz). Cube + JNI + fence pump can sit on that boundary. | Pin present to a fixed phase (always wait the *next* Choreographer timestamp, or count beats). Log consumed interval histogram. |
| H3 | `clocks.now` is process `Instant`, not `Choreographer.frameTimeNanos` | **Open** | Host `monotonic-clock.now` = `Instant::now` elapsed ns (`native/src/cm.rs`). Guest dt is “when GpuThread woke”, not the display vsync id. Small dt noise → micro-judder; large only if H1/H2 fire. | Pass vsync ns into guest only when WIT `frame-event` grows an instant (pin change — not this PR). |
| H4 | `queueWriteBuffer` **`ByteBuffer.allocateDirect` every call** | **Likely** | `DawnWasiWebGpuHost.queueWriteBuffer` allocates a direct buffer per uniform upload (cube: 64 B/frame). GC / allocator hitch on GpuThread. | Reuse a direct buffer; `synchronized(gpuLock)`. |
| H5 | `queueWriteBuffer` **does not take `gpuLock`** while `eventPoller` calls `processEvents` under that lock | **Open** | Write path is unsynchronized vs the 5 ms poller. Concurrent Dawn use on Mali is a known SIGSEGV class; can also hitch. | Hold `gpuLock` for `writeBuffer` (and other unlocked Dawn calls). |
| H6 | `PresentMode.Fifo` + `GPUSurface.present()` **does not wait** GPU/compositor on this AAR | **Open** | Configure hard-codes `PresentMode.Fifo` (`DawnWasiWebGpuHost` ~753). Device fact: Fifo `present()` returned without GPU completion (UAF if close too soon). CPU can present at a different phase than BLAST. | Confirm `getCapabilities().presentModes`; try Mailbox if advertised. Do not assume Fifo = vsync wait. |
| H7 | Fence callback delayed by **`POLL_MS = 5`** when not in `awaitCanvasGpuDone` | **Mitigated** (wait path) / **Open** | `awaitCanvasGpuDone` pumps `processEvents` every 1 ms. Background poller still sleeps 5 ms. Retire/other callbacks can still jitter 5 ms. | Pump or shorter poll while a canvas fence is in flight. |
| H8 | WG-6 **auto-present on `queue.submit`** then guest `context.present` | **Closed** (double present) | `presentPendingCanvasFrameLocked` is idempotent (pending cleared). Second present is a no-op. Not two presents per frame. | — |
| H9 | SurfaceView / BLAST extra buffering vs Choreographer | **Trace** | UI `Choreographer.doFrame` posts the gate; SurfaceView compositing is a second clock. No systrace this round. | `atrace` gfx/view + `Choreographer` vs `GPUSurface.present` timestamps. |
| H10 | CM `stream.read` / `callRunConcurrent` cost on GpuThread | **Open** | Pin `on-frame` is a sync `func`; `poll_produce` **blocks** the CM driver (no stackful async). Extra work on the same thread as present. | Measure read+host import ns; not done. |
| H11 | Per-frame guest `resource.drop` (texture/view/encoder/cb) | **Open** / **Guest** | CM dtor is table-only; Dawn `close` is host ring. Extra JNI/CM traffic every frame. | Defer drops; host already recycles swapchain. |
| H12 | 1-slot drop when guest is slow (`pending` already set) | **Mitigated** overlap with C5/H2 | Unconsumed beats still drop. If guest regularly exceeds one vsync, cadence is floor(refresh / N). | Same as H2 histogram. |
| H13 | Guest dt **50 ms clamp** | **Closed** (this hitch) | Clamp only on stalls ≥50 ms. User hitch is frequent small stutter, not 50 ms jumps. | — |
| H14 | First guest frame uses dummy **1/60 s** dt (`last_ns == 0`) | **Closed** (launch only) | One frame. User hitch is continuous. | — |
| H15 | Uniform `write_buffer` of a **column-major mat4** every frame | **Open** | Correct WebGPU; cost is H4 allocation more than the 64 B copy. | Fold into H4. |
| H16 | ART GC / JIT on `GpuThread` (`HandlerThread`) | **Trace** | Interpreter stacks were seen in older crash dumps (`ArtInterpreterToInterpreterBridge`). Debug builds hitch more. | Release minify; `atrace` am. |
| H17 | `canvasContextPresent` after submit (guest still calls it) | **Closed** | Same as H8; no-op. | — |
| H18 | Depth / MSAA / oversized swapchain | **Closed** | Cube skipped depth after `depth24plus`→RGBA8 fact. Swapchain is window size. | — |
| H19 | Two Choreographer callbacks (`postFrameCallback` from `doFrame`) | **Closed** | Standard vsync registration; one callback chain. Not a double tick by itself. | — |
| H20 | Product linker missing `monotonic-clock.now` | **Closed** | Registered in `native/src/cm.rs`; cube imports it. | — |
| H21 | `GPUSurface.getCurrentTexture` holds `gpuLock` and may **block** in Dawn/BLAST | **Likely** | `DawnWasiWebGpuHost.surfaceGetCurrentTexture` calls `getCurrentTexture()` under `gpuLock`. If BLAST has no free image, GpuThread stalls **and** the event poller cannot `processEvents`. Stacks with H1 (wait GPU, then maybe wait acquire). | Log acquire ns and `SurfaceGetCurrentTextureStatus`; Timeout/Suboptimal are hitch signals. |
| H22 | `onSubmittedWorkDone` executor is inline `Runnable::run` | **Closed** (not a second clock) | `callbackExecutor = Executor(Runnable::run)`: fence runs on whoever calls `processEvents` (GpuThread pump or poller). Not a double vsync. Concurrent close vs unlocked `writeBuffer` is H5. | Fold into H5. |
| H23 | UI `Choreographer` → JNI `postGfxVsync` → GpuThread condvar | **Open** | `Store.postGfxVsync` is JNI from the UI thread; `wait_take` is GpuThread. Usually sub-ms; UI jank delays the beat. | systrace `Choreographer#doFrame` vs native `post`. |
| H24 | Window / Surface refresh ≠ Choreographer 120 Hz | **Open** / **Guest** | Host always configures `PresentMode.Fifo` (H6). The out-of-tree `SurfaceView` may prefer 60 Hz while `doFrame` is 120. | Examples repo: `Surface.setFrameRate` / display mode. Log present vs `frameTimeNanos` Δ. |
| H25 | `deviceGetQueue` **inserts a new** HandleTable `Queue` every call, no `gpuLock` | **Open** | `DawnWasiWebGpuHost.deviceGetQueue`. Hitch only if guest `get-queue` per frame (table growth). Cube should cache. | Intern one queue handle; hold `gpuLock`. |
| H26 | `tryDrop` of old swapchain during `awaitCanvasGpuDone` `processEvents` | **Open** | Fence callback `retireGpuDoneCanvasFramesLocked` can run on the vsync-to-present pump. Mali `GPUTexture.close()` stalls that path. | Retire only on the poller after present, never during acquire wait. |
| H27 | Android 16 VRR / display-mode switch | **Trace** | V2458A is 120 Hz capable; the active mode can change. | `dumpsys display`; systrace. |
| H28 | Per-frame `textureCreateView` / encoder / bind-group CM+JNI | **Open** | Host `textureCreateView` and `deviceCreateCommandEncoder` take `gpuLock` (unlike H4). Cost is JNI/CM, not a Dawn race. Overlaps H10/H11. | Measure; defer guest drops (H11). |

## 3. Suggested kill order

1. **H1 + H26** — do not wait GPU (or close old BLAST images) between vsync and present; UAF ring stays.  
2. **H21** — log acquire time/status; do not hold `gpuLock` across a blocking `getCurrentTexture` if the AAR allows.  
3. **H4+H5** — lock + reuse direct buffer for `writeBuffer`.  
4. **H2** — device histogram of `on-frame` consume intervals (8.3 vs 16.6 ms).  
5. **H6+H9+H24** — present-mode + Surface frame rate vs Choreographer systrace.  
6. **H3** — only after WIT `frame-event` instant, or a documented host-only clock (not a second WIT).

## 4. What this branch already changed

- Product canvas swapchain ring + `onSubmittedWorkDone` (async) + wait **previous** fence on next acquire.  
- No 60 Hz `postGfxVsync` cap.  
- `in_frame` vsync drop in `native/src/host.rs`.  
- Cpu host pending/last canvas recycle; frame-lifetime instrument.  
- Out-of-tree cube: `clocks.now` dt (not in this git tree).

## 5. Device row

| Device | Hitch after C1–C5 | Notes |
|--------|-------------------|--------|
| Vivo V2458A (PD2415 / V2458A), Mali-G925, 120 Hz | **Yes** | Crash gone; visual hitch remains 2026-08-27. |
