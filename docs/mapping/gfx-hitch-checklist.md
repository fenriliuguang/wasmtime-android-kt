# Cube present hitch checklist

**English** | [中文](gfx-hitch-checklist.zh.md)

Not a cut queue. Living **device** investigation for the out-of-tree rotating cube (Vivo V2458A, Android 16, `arm64-v8a`, Mali-G925-Immortalis MC12, 120 Hz). Continuous `wasi-gfx` `on-frame`, not the 500 ms GFXV instrument. Do not vendor the demo. Do not file upstream GitHub issues.

Hitch **still observed** as a **~5 s small compositor rewind** after H12 made present 1:1 (~8.3 ms acquire, 0 `>20ms`). App-side DisplayManager votes that force SurfaceFlinger to 120 Hz **worsen** it (~3 s). H1–H28: do not restack those votes. **Far causes** (runtime wiring / guest / Wasmtime / compositor) are §6. Do not vendor the demo. Do not file upstream GitHub issues.

Related: [`gap-webgpu-wit-androidx.md`](gap-webgpu-wit-androidx.md) §5, [`threading-android.md`](threading-android.md) §8, [`frame-loop-suggestion.md`](frame-loop-suggestion.md). Guest clocks/dt lives in the **out-of-tree** examples repo. Dawn C A/B (same hitch after dropping androidx JNI): [`gfx-hitch-native-dawn.md`](gfx-hitch-native-dawn.md).

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
| C2 | `close()` same present / next acquire / CPU-frame ring | **Closed** (UAF) | Immediate close → `0x20` / `0x1f8`. CPU keep-last-N without fence → crash ~45 s. Keep last 3 **and** GPU done. Keep-8 exhausted the BLAST pool. |
| C3 | Host ~60 Hz cap on `postGfxVsync` | **Mitigated** | `MIN_GFX_VSYNC_NS` removed. Fast launch gone; hitch remains. Cap itself caused every-other-beat jitter on 120 Hz. |
| C4 | Guest `angle += const` per `on-frame` | **Mitigated** / **Guest** | Cube now uses `wasi:clocks/monotonic-clock#now` (rAF-style dt, 50 ms clamp). Pin `frame-event` is still `{ nothing: bool }`. |
| C5 | Consume a vsync that arrived mid-frame | **Mitigated** | Native `GfxOnFrameGate`: `post` drops if `pending \|\| in_frame`. Hitch remains → not the only cause. Native test `wasi_gfx_frame_loop_vsync_paced` updated. |
| C6 | GFXV instrument | **Closed** | `CLOSE_AFTER_VSYNC_MS = 500`; never saw seconds of present. Cpu recycle: `WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest`. |
| C7 | Per-frame `onSubmittedWorkDone` JNI global-ref leak (androidx.webgpu) | **Mitigated** (crash) | `GPUQueue.onSubmittedWorkDone` leaks a JNI global ref for **both** `callback` and `executor` on every call (never `DeleteGlobalRef`). 2 refs/frame → `global reference table overflow (max=51200)` `SIGABRT` on GpuThread ~3.5 min (25,320 frames). Dump: 25,312 × `ExternalSyntheticLambda4` (1 unique = the shared executor) + 25,308 × `queueSubmit$2` (the per-frame callback). Fencing every 2 frames halves the rate (**verified**: overflow moved 25,320 → ~50,640 frames, 3.5 → 7.0 min @120 Hz). Root fix is an upstream / self-hosted wgpu FFI — this AAR’s `libwebgpu_c_bundled.so` exports only `Java_androidx_webgpu_*`, no `wgpuQueueOnSubmittedWorkDone`. |
| C8 | H21 race: `processEvents` (locked) vs `getCurrentTexture` (unlocked) → Mali SIGSEGV | **Open** (crash, intermittent) | eventPoller `processEvents` → Dawn `vulkan::driver::QueueSubmit` → `libGLES_mali` null-pointer (`signal 11`, fault `0x0`). Hit once at 90 s; next run reached the C7 overflow at 423 s without it. Re-locking acquire re-blocks the poller (H21). | Investigate only if it recurs. |

## 2. Remaining causes (check in this order)

| ID | Hypothesis | Check | How we checked | Next if still Open |
|----|------------|-------|----------------|--------------------|
| H1 | After vsync, `get-current-texture` **waits previous GPU** (`awaitCanvasGpuDone`) so present is late vs scanout | **Mitigated** | Wait removed. Acquire is immediate (`last` 0.2–0.6 ms, `SuccessOptimal`). Ring+fence still recycle. | — |
| H2 | `in_frame` drop + 120 Hz: work time straddles 8.3 ms → **60/120 Hz oscillation** (classic judder) | **Mitigated** | Was: count 2 beats since last take (stable 60 Hz produce). V2458A mode 3 60 Hz + 60 fps still rewound BLAST (H27). Now **1:1** with Choreographer. | Visual on 120 Hz panel. |
| H3 | `clocks.now` is process `Instant`, not `Choreographer.frameTimeNanos` | **Mitigated** | Host-only: while `in_frame`, `now` is the vsync instant of the consumed beat (accumulate Choreographer dt on the WASI epoch). Not a WIT `frame-event` instant. | Visual re-check with H9. |
| H4 | `queueWriteBuffer` **`ByteBuffer.allocateDirect` every call** | **Mitigated** | Reuses one direct buffer; `synchronized(gpuLock)`. | Device: if hitch remains, not H4 alone. |
| H5 | `queueWriteBuffer` **does not take `gpuLock`** while `eventPoller` calls `processEvents` under that lock | **Mitigated** | `writeBuffer` holds `gpuLock`. | Fold leftover Dawn races into H21 if SIGSEGV returns. |
| H6 | `PresentMode.Fifo` + `GPUSurface.present()` **does not wait** GPU/compositor on this AAR | **Mitigated** | Mailbox at 120 Hz 1:1 grew BLAST to **6** and mixed 8/16 ms acquire. Back to Fifo + window buffer cap (H9). | Visual. |
| H7 | Fence callback delayed by **`POLL_MS = 5`** when not in `awaitCanvasGpuDone` | **Mitigated** | Acquire wait path gone. Poller sleeps 1 ms while a canvas fence is in flight, else 5 ms. | — |
| H8 | WG-6 **auto-present on `queue.submit`** then guest `context.present` | **Closed** (double present) | `presentPendingCanvasFrameLocked` is idempotent (pending cleared). Second present is a no-op. Not two presents per frame. | — |
| H9 | SurfaceView / BLAST extra buffering vs Choreographer | **Closed** (app cannot shrink) | Mailbox: **6** BLAST. `setBufferCount(3)` = EINVAL. `setBufferCount(4)` rc=0, gfxinfo still **5**. | Compositor pool is a floor with H27. |
| H10 | CM `stream.read` / `callRunConcurrent` cost on GpuThread | **Closed** (this 5 s hitch) | Would be continuous judder. After H12, Choreographer and acquire stay ~8.3 ms. | — |
| H11 | Per-frame guest `resource.drop` (texture/view/encoder/cb) | **Closed** (this 5 s hitch) | Same: extra JNI every frame, not a 5 s period. | — |
| H12 | Extra vsync wait after wait-start (`start_gen+1`) forced 60 fps | **Mitigated** | Unconsumed beats still drop. Latch `last_take_gen` to current gen (one present per stall). Device: acquire `lastDtNs` ~16 ms while Choreographer was 8.3 ms. | Histogram should move toward `<11ms`. |
| H13 | Guest dt **50 ms clamp** | **Closed** (this hitch) | Clamp only on stalls ≥50 ms. User hitch is frequent small stutter, not 50 ms jumps. | — |
| H14 | First guest frame uses dummy **1/60 s** dt (`last_ns == 0`) | **Closed** (launch only) | One frame. User hitch is continuous. | — |
| H15 | Uniform `write_buffer` of a **column-major mat4** every frame | **Mitigated** | Folded into H4 (scratch buffer). | — |
| H16 | ART GC / JIT on `GpuThread` (`HandlerThread`) | **Closed** (this 5 s hitch) | Reuse JNI `jbyteArray`. After H12: 0 `>20ms` acquires; app GC itvl **~87 s**, not 5 s. | Release minify only if alloc hitch returns. |
| H17 | `canvasContextPresent` after submit (guest still calls it) | **Closed** | Same as H8; no-op. | — |
| H18 | Depth / MSAA / oversized swapchain | **Closed** | Cube skipped depth after `depth24plus`→RGBA8 fact. Swapchain is window size. | — |
| H19 | Two Choreographer callbacks (`postFrameCallback` from `doFrame`) | **Closed** | Standard vsync registration; one callback chain. Not a double tick by itself. | — |
| H20 | Product linker missing `monotonic-clock.now` | **Closed** | Registered in `native/src/cm.rs`; cube imports it. | — |
| H21 | `GPUSurface.getCurrentTexture` holds `gpuLock` and may **block** in Dawn/BLAST | **Mitigated** | Acquire runs **without** `gpuLock` so the poller can `processEvents` + retire. Logs acquire ns and `SurfaceGetCurrentTextureStatus` (`GfxHitch`). Timeout/Suboptimal still hitch signals. | Device: watch `GfxHitch` acquire warnings. |
| H22 | `onSubmittedWorkDone` executor is inline `Runnable::run` | **Closed** (not a second clock) | `callbackExecutor = Executor(Runnable::run)`: fence runs on whoever calls `processEvents` (now the poller). Not a double vsync. | Fold into H5. |
| H23 | UI `Choreographer` → JNI `postGfxVsync` → GpuThread condvar | **Closed** (this 5 s hitch) | `FullscreenSurface` histogram: all `<11ms` while the ~5 s hitch is visible. | — |
| H24 | Window / Surface refresh ≠ Choreographer 120 Hz | **Guest** / **Mitigated** | Peak `preferredDisplayModeId` + content `setFrameRate` at that Hz (not 60). | `FullscreenSurface` vs Choreographer histogram. |
| H25 | `deviceGetQueue` **inserts a new** HandleTable `Queue` every call, no `gpuLock` | **Mitigated** / **Guest** | Host interns one queue handle under `gpuLock`. Cube already caches `device.queue()`. | — |
| H26 | `tryDrop` of old swapchain during `awaitCanvasGpuDone` `processEvents` | **Mitigated** | Fence callback only `countDown`. `retireGpuDoneCanvasFramesLocked` runs on the poller after `processEvents`, never during acquire. | Device: if close() still hitches, H16/H9. |
| H27 | Android 16 VRR / display-mode switch | **Closed** (app-side) | Forcing SF 120 → hitch **~3 s** (reverted). `CHANGE_FRAME_RATE_ONLY_IF_SEAMLESS` **still ~5 s**. Choreographer/acquire stay 8.3 ms through the pop. `mAlwaysRespectAppRequest=false`; Vivo `vivo_rms_screen`. **D22 2026-09-01:** Settings `min_refresh_rate=120` + `vivo_screen_refresh_rate_mode=120` (no app vote). | Do not restack DisplayManager / GameState. User lock stays on device; revert `settings delete system min_refresh_rate` and `vivo_screen_refresh_rate_mode=1`. |
| H28 | Per-frame `textureCreateView` / encoder / bind-group CM+JNI | **Closed** (this 5 s hitch) | Same as H10/H11: per-frame cost, not a 5 s rewind. | — |

## 3. Suggested kill order

Done on device. **Do not** restack DisplayManager / GameState / SurfaceControl votes.

1. **H12** — Landed: 1:1 present, overall smooth.  
2. **H27/H9** — Leftover **~5 s** is not an app vsync miss.  
3. **§6** — Far causes. Next probe is compositor `timestats` / FrameTimeline, not another Wasmtime pin.

## 4. What this branch already changed

- Product canvas swapchain ring + `onSubmittedWorkDone` (async); **no** GPU wait on next acquire; retire on the event poller after GPU done and **3** newer presents (H1/H26/C2).  
- `onSubmittedWorkDone` fences every **2** frames (batch) instead of every frame (C7: androidx.webgpu leaks a callback+executor global ref per call).  
- `getCurrentTexture` without `gpuLock`; acquire ns + interval histogram (`GfxHitch`) (H21/H2).  
- `queueWriteBuffer` holds `gpuLock` and reuses a direct buffer (H4/H5).  
- Prefer `PresentMode.Fifo` + `ANativeWindow_setBufferCount(4)` before configure (H9; count 3 = EINVAL, count 4 does not shrink gfxinfo BLAST below 5). Intern one device queue (H25).  
- No 60 Hz `postGfxVsync` cap.  
- `in_frame` vsync drop in `native/src/host.rs`; consume **every** Choreographer beat; latch stall to one present, no extra wait-start beat (H2/H12/H27).  
- Host-only: `clocks.now` during `on-frame` is that beat’s vsync instant (H3).  
- Cpu host pending/last canvas recycle; frame-lifetime instrument.  
- Out-of-tree cube: `clocks.now` dt (not in this git tree). Examples host: peak display mode + `setFrameRate` at that Hz + `setPreferMinimalPostProcessing` (H9/H24/H27).

## 5. Device row

| Device | Hitch after C1–C5 | Notes |
|--------|-------------------|--------|
| Vivo V2458A (PD2415 / V2458A), Mali-G925, 120 Hz | **overall smooth; ~5 s small hitch** | H12: acquire ~8.3 ms through the pop. Force SF 120 → ~3 s. Far causes: §6. 2026-08-27. |

## 6. Far causes after H12 (wiring / guest / Wasmtime / compositor)

**Filter.** The ~5 s pop happens while `FullscreenSurface` Choreographer and `GfxHitch` acquire stay ~8.3 ms (`>20ms=0`). Anything that **stalls** `run_concurrent`, GpuThread, MoonBit GC, or a Wasmtime epoch/fuel interrupt would spike `lastDtNs`. Those are **Closed** for this hitch. What remains must either (a) let the app keep presenting while SurfaceFlinger **rewinds stuffed BLAST images**, or (b) be an OEM policy that does not miss app vsync. **D24 correction (2026-08-29):** the pure-androidx.webgpu native cube shows the same clean timing but **no ~5 s pop**, so (a)/(b) do not reproduce the hitch alone — it is specific to the Wasmtime present path. **D25 (2026-09-01):** Dawn C NativeGpu (no androidx JNI) still pops → ART/Kotlin is not the amplifier. Hitch-branch probes then quantified **D2 phase** (~19% of vsync→present samples cross the 8.3 ms beat on the JNI path) and **Confirmed D3** (no present timestamp; SF stuffing). After D25 the remaining chain is guest + CM pump + untimestamped `present` + BLAST. Transfer: [`gfx-hitch-native-dawn.md`](gfx-hitch-native-dawn.md).

Sources: this tree (`native/src/engine.rs`, `native/src/cm.rs` `poll_produce` / `nativeCallRunConcurrent`, `host-dawn` present, out-of-tree `guests/rotating-cube/gen/world/guest/run.mbt`); [AOSP frame pacing](https://source.android.com/docs/core/graphics/frame-pacing) (SF holds until in-phase); [AOSP game loops / buffer stuffing](https://android.googlesource.com/platform/docs/source.android.com/+/master/en/devices/graphics/arch-gameloops.html); [Perfetto FrameTimeline](https://perfetto.dev/docs/data-sources/frametimeline) (`BUFFER_STUFFING`, `PREDICTION_ERROR` “corrects itself periodically”); [Wasmtime interrupting execution](https://docs.wasmtime.dev/examples-interrupting-wasm.html) (epoch/fuel — **not** enabled here). Do **not** file upstream GitHub issues; record Android facts here.

### 6.1 Runtime wiring

| ID | Hypothesis | Check | How we checked | Next if still Open |
|----|------------|-------|----------------|--------------------|
| D1 | Sync pin `on-frame` + blocking `poll_produce` (no `Poll::Pending`) | **Closed** (this 5 s hitch) | `GfxOnFrameProducer` waits the condvar on GpuThread because the pin is a sync `func` and WAT traps on stream.read `BLOCKED`. That is **every vsync**, and it is why acquire is 8.3 ms, not a 5 s timer. | Stackful CM async is a product later; not this pop. |
| D2 | UI vsync → JNI `postGfxVsync` → 8MiB `wasmtime-cm-pump` → Dawn present | **Likely** — present **phase** (not stall) | Hitch branch: per-frame `vsync→present` (`System.nanoTime − frameTimeNanos`). Native androidx cube: **1.2–5.6 ms, 0% >8.3 ms** (0/4920). Wasmtime/`host-dawn`: **1.0–9.3 ms, ~19% >8.3 ms** (1427/7320). Acquire **interval** stays 8.3 ms — that is why D2 was wrongly Closed as stall. Split: `wakeA` 0.5–1.3 ms stable (condvar ok); real work ≈ wakeA 1 + guest 2.8 (MoonBit async/WIT-ABI 2.6 + matrix 0.2) + host imports 3.3 ≈ **7.1 ms < 8.33 ms beat**. The ~5 ms “guest compute” was a `frames.read(1)` wait artifact (~2.0 ms). `queueSubmit` 1.4 ms is auto-`present` ~0.85 ms (H8), not an anomalous submit. **D25:** cutting the 3.3 ms JNI imports does **not** remove the pop. | Dawn C: NativeGpu `GfxHitch` `present n=` `>8.3ms=` (N4/D2). Guest/host micro-opt will not remove the rewind; D3 is the compositor half. **P2/P3 (2026-09-01, 95 s / 12000 frames, Dawn C):** present `phase-crossing` **2** (margin −0.4 / −2.0 ms, **55.5 s apart**, ultra-slow beat drift); retire age `<8.3ms=0` all frames. CM pump phase lock + host present/buffer lifecycle are **clean** — the out-frame path is not this hitch. |
| D3 | Dawn `present()` has **no** presentation timestamp (no Swappy / `eglPresentationTimeANDROID`) | **Confirmed** (static) + **Mitigated** (fix 2026-09-01) | Host never sets a target present time. Hitch branch: `llvm-readelf`/`nm` on `libwebgpu_c_bundled.so` — NEEDED only android/log/dl/c/m, **no Swappy**, no `eglPresentationTimeANDROID`. SF timestats: `appBufferStuffingJankyFrames` dominant (21961 @60 + 34667 @90; `sfPredictionErrorJankyFrames` 4247). 20 s Perfetto display-level **2407/2407 On-time**. Vendor `debug.sf.disable_backpressure=1`. Rewind is layer/buffer, not display. D25: Dawn C `wgpuSurfacePresent` still has no timestamp. **Dawn C timestats 2026-09-01 ~45 s** (`--timestats -clear -enable` then `-dump`): BLAST SurfaceView **5** + 1 VRI. Timeline `appBufferStuffingJankyFrames=0` because `totalTimelineFrames=0` (SurfaceView not in FrameTimeline). Legacy histograms still show stuffing **behavior**: `desired2present` / `acquire2present` bimodal ~18–19 ms vs ~28–30 ms (~41% extra vsync); `present2present` 8 ms=5398 / 33 ms=1; `present2presentDelta` 25 ms=2. **Fix 2026-09-01:** `ANativeWindow_setBuffersTimestamp` immediately before `wgpuSurfacePresent` (default **2** Choreographer beats; `debug.wasmtime.gfx.desired_present_beats` / `WASMTIME_GFX_DESIRED_PRESENT_BEATS`, `0` off). V2458A `rc=0`. ~59 s 1:1 with D22 lock: `desired2present` **5 ms=7072** (was parked ~28–30 ms); `present2presentDelta` 25 ms=0; `averageFPS` 125.062 (skip `n=6` was 104). | A/B: `desired_present_beats 0`. Skip-present stays a probe. No Mailbox. |
| D4 | Dual present (`queue.submit` auto-present + guest `ctx.present`) | **Closed** | H8: second present is a no-op. | — |
| D5 | Event poller `POLL_MS` 5 / fence retire keep-3 | **Closed** (this 5 s hitch) | 5 ms sleep is not a 5 s period. Keep-3 is ~25 ms of GPU lifetime, not compositor rewind. | — |
| D6 | `clocks.now` wiring vs Choreographer | **Closed** (this 5 s hitch) | H3: in-frame `now` is the consumed vsync instant. A compositor rewind still **shows an old image** even when guest dt is correct. | — |
| D7 | JNI / ART GC on the pump | **Closed** (this 5 s hitch) | H16: app GC itvl ~87 s; 0 `>20ms` acquire. | — |

### 6.2 Guest (out-of-tree MoonBit cube)

| ID | Hypothesis | Check | How we checked | Next if still Open |
|----|------------|-------|----------------|--------------------|
| D8 | MoonBit `async-core` scheduler / extra coroutines | **Closed** (this 5 s hitch) | `run` uses `surface.on_frame()` then `frames.read(1)` in a `for`. Host drives that as CM `run: func() -> u32` via `call_concurrent`. Extra guest yields would delay acquire. Histogram is clean. | — |
| D9 | `frames.read(1)` 0-length readiness then skip | **Closed** (this 5 s hitch) | `chunk.length() == 0` → `continue` (no present). That is a **skipped** app frame (histogram hole), not a 5 s rewind of old BLAST. | If 0-length reads appear in a future guest, log skip count. |
| D10 | Per-frame `create_view` / encoder / `drop` of WIT owns | **Closed** (this 5 s hitch) | Needed so Wasmtime’s resource table does not grow until `nativeCallRunConcurrent` SIGSEGV. Host swapchain recycle is table-only on `color_tex.drop`. Cost is every frame. | Guest-side reuse of encoder/view is a size/CPU win, not a 5 s fix. |
| D11 | `frame_delta_sec` 50 ms clamp / first-frame 1/60 | **Closed** | H13/H14. User pop is a 1–2 frame rewind, not a 50 ms angle jump. | — |
| D12 | Pin `frame-event` is `{ nothing: bool }` (no rAF timestamp in WIT) | **Mitigated** / **Guest** | Guest uses `monotonic-clock.now` instead. WIT cannot carry Choreographer ns until pin changes. | Not this hitch if acquire is 1:1. |
| D13 | Guest angle vs compositor rewind | **Closed** (keep-6 worsens) / **Open** (eye) | If SF re-shows BLAST *n−2*, the cube **looks** like it went back even though `angle` already advanced. D3 timestamp drained `desired2present` (~29→5 ms) but the pop remains. keep-6 **increased** hitch frequency; SurfaceView BLAST stayed **5**. Reverted to keep-3. Same class as keep-8 / H27. **P4 (2026-09-01, 95 s / 12720 frames):** guest angle clock (Choreographer `frameTimeNanos` per-frame delta) `angleDt 8-9ms=all`, `9-17ms=0`, `>17ms=0` — the transform matrix is **strictly linear in time**, no skip, no rewind. **N9 (2026-09-01):** present submission details (configure / Dawn commit / 65 s SF timestats) are clean — no rewind/drop/jank on the Dawn C path. | Do not restack keep. |

### 6.3 Wasmtime (this Engine)

| ID | Hypothesis | Check | How we checked | Next if still Open |
|----|------------|-------|----------------|--------------------|
| D14 | Epoch interruption or fuel (periodic yield/trap) | **Closed** | `native/src/engine.rs`: only `wasm_component_model` + `wasm_component_model_async`. No `epoch_interruption`, no fuel. [Wasmtime interrupt docs](https://docs.wasmtime.dev/examples-interrupting-wasm.html) do not apply until we opt in. | Do not enable epoch on the cube Store. |
| D15 | Cranelift / pooling allocator / memory grow every ~5 s | **Closed** (this 5 s hitch) | Compile/instantiate is once. Guest is a tight loop with fixed buffers (864 + 64 byte writes). A grow or compiler pause would stall acquire. | — |
| D16 | CM async `run_concurrent` internal timer | **Closed** (this 5 s hitch) | Pump is `pollster::block_on` of one `call_concurrent("run")` for the whole session. No 5 s host timer in this wrapper. | — |
| D17 | Resource-table compaction / dtor storms | **Closed** (this 5 s hitch) | Guest drops 5 owns per frame **on purpose**. Unbounded table was a crash, not a 5 s hitch. Dtors are table-only for swapchain. | — |
| D18 | “Wasmtime is slow / CM is experimental” as the 5 s period | **Closed** (this hitch) | CM async is why `on-frame` exists at all. Slow CM would be **continuous** 60/120 judder (we had that before H12). After H12 the period is compositor-shaped. | Do not file Wasmtime issues. |

### 6.4 Compositor / OEM (web + device)

| ID | Hypothesis | Check | How we checked | Next if still Open |
|----|------------|-------|----------------|--------------------|
| D19 | 5 BLAST BufferQueue stuffing; SF re-shows then drains | **Mitigated** (timestamp) | Same as D3. Hitch branch timestats confirmed stuffing as the dominant *jank class*. Dawn C: Timeline class **n/a** (`totalTimelineFrames=0`); `desired2present` bimodal was the stuffing signal. Skip `n=6` drained the extra vsync (FPS ~104). **Timestamp beats=2** drained `desired2present` to ~5 ms at FPS 125. | Skip remains a probe. No Mailbox. |
| D20 | SF “hold until in phase” with a throttled UID FPS (divisor of 120) | **Closed** (2026-09-01) | `dumpsys SurfaceFlinger` (V2458A, Android 16): `GameFrameRateOverrides=` empty (no GameManager/RMS UID mapping); `setFrameRate=(uid, frameRate)={10504, 120.00 Hz}`; SurfaceView BLAST layer `requestedFrameRate: {120.00 Hz ExactOrMultiple}`; `idleScreenConfig timeoutMillis:-1`; that layer's `FPS ring buffer` stayed 120.39/120.31/120.53… (no 60/90 bin). SF does not throttle this UID to a divisor of 120. | Do not GameState / DisplayManager votes. |
| D21 | VSyncPredictor `PREDICTION_ERROR`, scheduler “corrects periodically” | **Closed** (this hitch) | Predictor recovery **off** (`vsync_predictor_recovery: false`; `compositionStrategyPredictionState=DISABLED`). Hitch-branch display Perfetto 2407/2407 On-time. Dawn C 2026-09-01 ~22 s: **2155/2155** actual display frames `JANK_NONE` / `PRESENT_ON_TIME` / `PREDICTION_VALID`; **0** `JANK_PREDICTION_ERROR`. SurfaceView BLAST emits **no** surface-frame timeline (Perfetto: SurfaceViews unsupported; matches D3 `totalTimelineFrames=0`). A stuffed BLAST rewind would not show as prediction-error jank. | — |
| D22 | Vivo RMS / `vivo_rms_screen` / adaptive 60·90·120 | **Closed** (Hz dip as remaining cause) / **Mitigated** (Settings lock this window) | Device has `vivo_rms_screen`; `mAlwaysRespectAppRequest=false`; forcing SF 120 **increased** hitch frequency (H27). **2026-09-01 Settings lock** (no app vote): `min_refresh_rate` null→`120.0`, `vivo_screen_refresh_rate_mode` 1→120, `peak_refresh_rate` stayed 120. DisplayModeDirector `PRIORITY_USER_SETTING_MIN_RENDER_FRAME_RATE` 0→120. ~60 s 1:1 TimeStats vs D3 smart-switch baseline: `refreshRateSwitches=0`; `averageFPS=125.007`; `present2present` 8 ms=7124 (no 16/25/33 ms); `present2presentDelta` 25 ms **0** (was 2); `desired2present` **unimodal** ~28–30 ms (was bimodal 18–19 vs 28–30). BLAST still **5**. `GfxHitch` `lastDtNs` still ~8.3 ms. Lock parked stuffing depth; it did not drain the extra vsync (that was skip `n=6`). | Timestamp landed (D3). Do not restack votes. Revert: `settings delete system min_refresh_rate`; `settings put global vivo_screen_refresh_rate_mode 1`. |
| D23 | Kernel / DisplayModeDirector idle refresh timeout | **Open** (weak) | Cube presents every vsync, so the layer is not idle. Idle timers (often seconds) still appear in vendor SF. | Overlay “show refresh rate”; watch if Hz dips at the pop. |
| D24 | Native androidx.webgpu cube (no Wasmtime) would hitch the same | **Closed** — upstream does **not** reproduce it alone | `hosts/native-webgpu` `CubeActivity` (pure androidx.webgpu 1.0.0-alpha05, no Wasmtime, no `host-dawn`) on V2458A: 1:1 present @120 Hz, Choreographer ~8.3 ms (0 `>20ms`), acquire ~8.3 ms (0 `>20ms`, `SuccessOptimal`), display pinned 120 Hz — clean timing **and no ~5 s pop** (eye-checked). Upstream androidx.webgpu / SF / OEM VRR is smooth on its own. | The pop is Wasmtime/`host-dawn`-specific: re-open D2 (CM pump vsync→present latency) and D3 (no present timestamp). |
| D25 | NativeGpu Dawn C + Wasmtime would be smooth (no androidx JNI) | **Closed** — same ~5 s class | 2026-09-01 `fullscreen-surface` + `libwebgpu_dawn.so`: `dlopen` ok, Vulkan adapter, Choreographer 120 Hz 0 `>20ms`, visual rewind remains. ART/Kotlin hot-path tax is **Closed** for this hitch. | Transfer table: [`gfx-hitch-native-dawn.md`](gfx-hitch-native-dawn.md). |

### 6.5 Suggested probes (one variable)

1. **`dumpsys SurfaceFlinger --timestats`** — **Done** hitch branch (androidx, stuffing *class*). **Done** Dawn C 2026-09-01: must `-clear -enable` then `-dump` (bare `--timestats` is empty). Timeline class 0; `desired2present` bimodal ~one extra vsync.
2. **Perfetto FrameTimeline** ~20 s — **Done** hitch branch (display all On-time). **Done** Dawn C 2026-09-01: 2155/2155 display On-time + Valid; 0 PredictionError; SurfaceView layer timeline n/a.
3. **Settings lock 120 Hz** (no app vote). → **Done** (D22 2026-09-01): `min_refresh_rate=120` + `vivo_screen_refresh_rate_mode=120`. 25 ms rewind deltas 2→0 this ~60 s; `desired2present` parked at stuffed ~29 ms. BLAST still 5.
4. **Skip-present every N** — **Done** `n=6` A/B: extra-vsync `desired2present` cluster gone; 25 ms `present2presentDelta` 2→0. Gate default off (`debug.wasmtime.gfx.skip_present_n`). No Mailbox.
5. **No-Wasmtime native cube** A/B. → **Done** (D24): smooth. Hitch is Wasmtime-path-specific.
6. **Dawn C NativeGpu A/B.** → **Done** (D25): same pop → not the androidx façade.
7. **N4 + D2 histograms on NativeGpu** (`GfxHitch` acquire interval + `vsync→wgpuSurfacePresent`). → **Done** (2026-09-01 Dawn C): acquire `>20ms=0`; `>8.3ms=0` (hitch JNI path was 19%). Stall Closed; cross-beat phase Closed on this sample. BLAST still 5.

### 6.6 Bottleneck (hitch-branch numbers × D25)

Not a single slow hotspot. Work fits the beat (~7.1 ms in 8.33 ms on the JNI path). Two stages:

1. **Phase source (D2, mid-chain).** CM pump serializes guest execution into the present deadline. Condvar (`wakeA`) is stable. D25 closed the 15 JNI GPU imports as *necessary* for this 5 s pop. Remaining jitter is guest async/WIT-ABI. **P2/P3 (2026-09-01, 95 s / 12000 frames):** `phase-crossing` 2 (55.5 s apart); retire `<8.3ms=0`. **P4 (2026-09-01, 95 s / 12720 frames):** guest angle clock per-frame delta `angleDt 8-9ms=all`, `9-17ms=0`, `>17ms=0` — the transform matrix is **strictly linear in time**. **P5 (2026-09-02, ~12 min):** `take-skip` **0** — the guest `now`/`angle` steps exactly one 8.33 ms beat, **decoupled from present phase**. **Phase addendum (2026-09-02):** re-reading the full `phase-crossing` sequence found present phase can **snap out of lock** — once (23:44), `lat` 5→9.6 ms, `margin` +3→−1.3 ms in one beat, then `cross` several/s; seen **once in 30 min**, and P5 shows it does **not** propagate into guest content. The out-frame path (CM pump phase + present/buffer lifetime) and guest content are both **clean**.
2. **Where it becomes a rewind (D3/D19, end of chain).** **Mitigated:** `ANativeWindow_setBuffersTimestamp` (default 2 beats) dropped `desired2present` ~29 ms → ~5 ms at 125 fps. BLAST is still 5. D13 if a visual rewind remains. **D20 Closed (2026-09-01):** SF has no GameFrameRateOverrides, UID explicitly 120 Hz, idle timeout -1, FPS ring stable at 120 — no SF-side throttle. **N9 (2026-09-01):** the present submission details are clean — `surface-caps formats=[22,23,40,30] present=[4,1] alpha=[4]` (RGBA8 + `Fifo(1)` + `Inherit(4)`), configure aligned to `caps.alphaModes[0]`, Dawn pin = androidx AAR SHA (same commit), and a 65 s SF window (`droppedFrames=0`, `jankyFrames=0`, `present2presentDelta 25ms=0`, `desired2present 5ms=7811`) shows **no rewind/drop/jank** on the Dawn C path.

**Conclusion (2026-09-02, verification paused):** all three observable layers — guest content (P4/P5 linear, decoupled), SF counters (`droppedFrames=0`, `jankyFrames=0`, no rewind/drop), and present submission (N9 clean) — are **clean**. The visual pop therefore sits in an un-measured layer: most likely the compositor/panel BLAST re-show (D13/D19) outside SF's counters, or the rare present-phase snap's host-side transient — neither yet frame-captured at the exact pop instant. Next needs event-triggered capture (screenrecord keyed on `sinceLast` collapse) or the real `onSubmittedWorkDone` fence lane to close the last D24 structural gap.

Hitch-branch fix ranking after D25: (1) present timestamp / `setDesiredPresentTime` — **landed** (`ANativeWindow_setBuffersTimestamp`, default 2 beats), (2) draw/present split, (3) cut JNI — **already done, insufficient**, (4) cut condvar — low, (5) simplify guest — shrinks D2 only. Skip-present is a stuffing probe, not a fix.

Living Dawn C table: [`gfx-hitch-native-dawn.md`](gfx-hitch-native-dawn.md).
