# Cube hitch: Dawn C vs androidx.webgpu

**English** | [中文](gfx-hitch-native-dawn.zh.md)

Not a cut queue. Not a native-dawn consume lane. Device A/B after NativeGpu wired `wgpu*` (2026-09-01). Do not vendor the demo. Do not file upstream GitHub issues. Named-only hitch work (D3 present timestamp / skip-present) stays named — see [`../agent/native-dawn.md`](../agent/native-dawn.md).

androidx + `host-dawn` JNI table: [`gfx-hitch-checklist.md`](gfx-hitch-checklist.md). IDs **C / H / D** are reused so we do not renumber. New Dawn C rows are **N\***.

Same device as the androidx table: Vivo V2458A (PD2415), Android 16, `arm64-v8a`, Mali-G925-Immortalis MC12, 120 Hz. Host: out-of-tree `hosts/fullscreen-surface`. Guest: same MoonBit rotating cube.

**Check**

| Tag | Meaning |
|-----|---------|
| **Open** | Still a plausible hitch on the Dawn C path; not disproved |
| **Likely** | Strongest remaining suspect after the A/B |
| **Mitigated** | Invariant already landed; hitch remains so not sufficient alone |
| **Closed** | Ruled out for *this* ~5 s pop (may still be a crash/leak fact) |
| **Trace** | Needs a histogram / Perfetto / timestats on this path; not done yet |
| **n/a** | Does not exist on Dawn C (androidx-only) |

## 0. A/B triangle (2026-09-01)

| Build | GPU consume | Wasmtime / CM pump | ~5 s rewind |
|-------|-------------|--------------------|-------------|
| `hosts/native-webgpu` `CubeActivity` | androidx.webgpu JNI only | no | **No** (D24, eye-checked 2026-08-29) |
| `fullscreen-surface` + `host-dawn` | androidx JNI via `ExperimentalHostCallbacks` | yes | **Yes** (~5 s; Choreographer / acquire stay ~8.3 ms) |
| `fullscreen-surface` + NativeGpu Dawn C | `dlopen` `libwebgpu_dawn.so` + `wgpu*` (no androidx on the cube hot path) | yes | **Yes** (same visual class; Choreographer 120 Hz, 0 `>20ms`) |

**Filter.** Bypassing ART / Kotlin / androidx JNI on the GPU hot path **does not** remove the pop. The native-dawn “Why” (phase jitter amplified by frequent ART crossings) is **Closed for this hitch**. What is still shared is: guest + `GfxOnFrameGate` / `postGfxVsync` + 8MiB `wasmtime-cm-pump` + hitch ring (keep-3 / Fifo / H8) + SurfaceView BLAST + OEM VRR. Present timestamp landed (D3). keep-6 **worsened** hitch frequency (reverted).

Choreographer on the Dawn C install stayed `<11ms` through minutes of 120 Hz beats (same shape as H23). That still **closes stall-type** causes (CM trap, epoch, MoonBit GC pause, acquire wait). It does **not** close a compositor rewind of stuffed BLAST images, or a vsync→present **phase** that never spikes `lastDtNs`.

## 1. Transfer: androidx rows → Dawn C

Reuse the androidx evidence unless the Dawn C path changed the fact. “androidx Check” is from [`gfx-hitch-checklist.md`](gfx-hitch-checklist.md).

### 1.1 Closed or n/a (do not rediscover)

| ID | Hypothesis | androidx | Dawn C | Transfer |
|----|------------|----------|--------|----------|
| C1 | Leak swapchain textures → SIGSEGV | Closed (crash) | keep-3 + deferred `wgpuTextureRelease` | **Closed** (crash). Cube ran minutes without `0x20`. |
| C2 | `close()` on the just-presented image | Closed (UAF) | same invariant | **Closed** (UAF). |
| C3 | Host ~60 Hz `postGfxVsync` cap | Mitigated | same `host.rs` | **Mitigated**. Shared. |
| C4 | Guest `angle += const` | Mitigated / Guest | same wasm | **Mitigated** / **Guest**. |
| C5 | Drop vsync while `in_frame` | Mitigated | same `GfxOnFrameGate` | **Mitigated**. Shared. |
| C6 | GFXV 500 ms instrument | Closed | not this host | **Closed**. |
| C7 | androidx `onSubmittedWorkDone` JNI global-ref leak | Mitigated (crash) | no AAR callback | **n/a**. Do not copy the 2-frame fence batch. |
| C8 | `processEvents` vs unlocked `getCurrentTexture` Mali race | Open (crash) | `process_events` + acquire on the **same** pump thread | **Closed** (this race). New Dawn-thread races are **N3**. |
| H1 | Acquire waits previous GPU fence | Mitigated | no wait on acquire | **Mitigated**. |
| H4 / H5 / H15 | Per-frame `ByteBuffer.allocateDirect` / unlocked `writeBuffer` | Mitigated | guest `list<u8>` → one host copy → `wgpuQueueWriteBuffer`; no ART | **Closed** (this hitch). Path gone; pop remains. |
| H8 / H17 / D4 | `queue.submit` auto-present + guest `context.present` | Closed | same H8 no-op | **Closed**. |
| H10 / H11 / H28 | Per-frame CM + **JNI** cost | Closed (5 s) | CM remains; JNI on GPU methods is gone | **Closed** (5 s). Would be continuous judder. |
| H13 / H14 / D11 | 50 ms dt clamp / first-frame 1/60 | Closed | same guest | **Closed**. |
| H16 / D7 | ART GC / JIT on `GpuThread` | Closed (5 s; GC ~87 s) | GPU hot path is Rust + Dawn C | **Closed** (stronger). A/B: less ART, same pop. |
| H18 | Depth / MSAA / oversized swapchain | Closed | cube still skips depth | **Closed**. |
| H19 | Two Choreographer callbacks | Closed | same host | **Closed**. |
| H20 | Missing `monotonic-clock.now` | Closed | same linker | **Closed**. |
| H22 | Inline fence executor as a second clock | Closed | no androidx executor | **n/a**. |
| H23 | UI Choreographer → JNI vsync → GpuThread condvar **stall** | Closed (5 s) | same vsync JNI; histogram still `<11ms` | **Closed** (stall). Phase is **D2**. |
| H25 | New `device.queue` handle every call | Mitigated | intern one queue | **Mitigated**. |
| D1 | Sync `on-frame` + blocking `poll_produce` | Closed (5 s) | unchanged | **Closed** (5 s). Every-beat cost, not a 5 s timer. |
| D8–D10 / D14–D18 | Guest scheduler / Wasmtime epoch / “CM is slow” | Closed | unchanged guest + Engine | **Closed**. Shared; histogram clean. |
| N7 | `wgpuInstanceWaitAny` 2 s on boot | — | adapter/device only | **Closed**. Not per-frame. |

### 1.2 Still live after the A/B (check in this order)

| ID | Hypothesis | androidx | Dawn C | Next |
|----|------------|----------|--------|------|
| **N1** | ART / Kotlin / androidx JNI on the GPU hot path is the 5 s pop | (playbook Why) | **Closed** | Do not chase more JNI removal for this hitch. |
| **D2** | Vsync → `postGfxVsync` → 8MiB CM pump → Dawn present **phase** (no `lastDtNs` spike) | Closed as *stall*; hitch branch **Likely** (~19% >8.3 ms on JNI path) | **Open** (in-beat wander) / **Closed** (cross-beat on this 25 s) | NativeGpu `GfxHitch` 2026-09-01: `present n=2880` `>8.3ms=0` (JNI path was 19%). `lastLatencyNs` still 0.8–6.9 ms inside the beat. Cross-beat phase is not the Dawn C story. D3 remains. **P2 (2026-09-01, 95 s / 12000 frames):** `phase-crossing` **2** (margin −0.4 / −2.0 ms, **55.5 s apart**) = ultra-slow beat drift (~1.25 ns/beat), **not** ~5 s. CM pump phase lock is clean. |
| **D3** / **D19** | `present()` has no presentation timestamp; BLAST stuffing; SF re-shows *n−2* | **Confirmed** (static + timestats) on androidx `.so` | **Mitigated** (timestamp beats=2) | Shared. **2026-09-01:** `ANativeWindow_setBuffersTimestamp` before `wgpuSurfacePresent` (`rc=0`). Default 2 beats. ~59 s with D22 lock: `desired2present` **5 ms=7072** (was parked ~29 ms); FPS 125.062. Skip `n=6` remains a probe. |
| **D13** | Cube *looks* like it rewinds while `angle` already advanced | Likely (symptom) | **Closed** (keep-6) / **Open** (eye) | Timestamp drained `desired2present` but the eye still pops. keep-6 (BLAST+1) **increased** hitch frequency; gfxinfo SurfaceView BLAST stayed **5**. Reverted to keep-3. Same class as keep-8 starving / H27 votes. **P4 (2026-09-01, 95 s / 12720 frames):** guest angle clock per-frame delta `angleDt 8-9ms=all`, `9-17ms=0`, `>17ms=0` — the transform matrix is **strictly linear in time**. The rewind can only come from SF re-showing old BLAST. | Do not restack keep. Next is not more images in flight. |
| **D20** | SF hold-until-phase + UID FPS divisor | Open | **Closed** (2026-09-01) | Shared. `dumpsys SurfaceFlinger` (V2458A / Android 16): `GameFrameRateOverrides=` empty; `setFrameRate UID=10504 → 120.00 Hz`; BLAST layer `requestedFrameRate 120.00 Hz ExactOrMultiple`; `idleScreenConfig timeout:-1`; layer `FPS ring buffer` stable 120. SF does not throttle to a divisor of 120. Do **not** GameState. |
| **D21** | VSyncPredictor periodic correction | Open / Trace | **Closed** (this hitch) | `vsync_predictor_recovery: false`. Dawn C 22 s Perfetto: **2155/2155** display frames `NONE` / `ON_TIME` / `VALID`. 0 `PREDICTION_ERROR`. SurfaceView BLAST has **no** surface-frame timeline (same n/a as D3). |
| **D22** / **H27** | Vivo RMS / smart refresh; app votes worsen it | Open / Likely; app votes Closed | **Closed** (Hz dip) / **Mitigated** (Settings lock) | **2026-09-01:** `min_refresh_rate=120` + `vivo_screen_refresh_rate_mode=120`. `present2presentDelta` 25 ms 2→0 this ~60 s; `desired2present` parked ~29 ms (was bimodal). BLAST still 5. Do not restack votes. |
| **D23** | Idle refresh timeout | Open (weak) | **Open** (weak) | Overlay Hz at the pop. |
| **D24** | No-Wasmtime androidx cube hitches the same | **Closed** (it does not) | still the control | Keep as the smooth baseline. |
| **D25** | Dawn C + Wasmtime cube is smooth | — | **Closed** (it is not; this page) | Hitch is not the androidx façade. |
| **H2** / **H12** | 60/120 every-other-beat / extra wait-start | Mitigated | same gate | **Mitigated**. Confirm Dawn C acquire histogram (**N4**). |
| **H3** / **D6** / **D12** | `clocks.now` vs Choreographer; pin `frame-event` has no timestamp | Mitigated | same | **Mitigated**. Compositor rewind still shows an old image. |
| **H6** / **H9** | Fifo vs Mailbox; BLAST floor 5 | Mitigated / Closed (app cannot shrink) | Fifo + `setBufferCount(4)` intent | **Mitigated**. Confirm gfxinfo BLAST count on Dawn C (**N5**). |
| **H21** | `getCurrentTexture` blocks under `gpuLock` | Mitigated | no `gpuLock`; `process_events` then `GetCurrentTexture` on the pump | **Mitigated** (lock). Blocking inside Dawn is **N3**. |
| **H26** / **D5** | Poller / keep-3 as a 5 s period | Closed (5 s) | keep-3 remains; fence is **not** a real GPU callback (**N2**) | **Closed** (5 s period). Lifetime is **N2**. |

## 2. Dawn C–only rows

| ID | Hypothesis | Check | Evidence | Next |
|----|------------|-------|----------|------|
| N1 | Removing androidx JNI / ART from the cube GPU path removes the pop | **Closed** | Same host + guest + vsync; only consume changed; pop remains | — |
| N2 | `mark_canvas_gpu_done()` on `queue.submit` is immediate; no `wgpuQueueOnSubmittedWorkDone` | **Open** (lifetime) / **Closed** (5 s timer) | Marks the whole ring `gpu_done` then keep-3 retires. keep-6 **worsened** hitch frequency (reverted). | Do not raise keep. Wire C-API work-done only on UAF. **P3 (2026-09-01, 95 s / 12000 frames):** retire age `<8.3ms=0` all frames; retire lands 24–28 ms (~3 beats). No pre-composite recycle — the `gpu_done` "lie" does **not** reuse a buffer under SF. |
| N3 | `wgpuInstanceProcessEvents` on acquire (pump thread) vs androidx poller | **Closed** (this 25 s) / **Open** (rare warn) | Same thread as present. One acquire warn `2031693ns status=1` at start; later acquire `last` 0.07–0.8 ms, `>20ms=0`. | — |
| N4 | Dawn C acquire / present-interval + vsync→present histogram | **Closed** (stall) | 2026-09-01 V2458A, `fullscreen-surface` NativeGpu, ~25 s: Choreographer 3000 `<11ms` `>20ms=0`. Acquire n=3000 `<11ms=2998` `11-20ms=2` `>20ms=0` `status=1`. Present interval n=2880 `<11ms=2867` `11-20ms=13` `>20ms=0`. | Stall-type stays Closed through the window. |
| N5 | `wgpuSurfacePresent` vs androidx `GPUSurface.present` stuffing | **Mitigated** (timestamp) / Timeline **n/a** / skip **drained** | gfxinfo: **5** SurfaceView BLAST Consumer (same H9 floor) + 1 VRI. 2026-09-01 1:1 TimeStats ~45 s: `desired2present` bimodal ~18–19 vs ~28–30 ms (~41% extra vsync); `present2presentDelta` 25 ms=2. Skip `n=6`: FPS 104.194; 28–30 ms cluster **0**. **D22 lock:** parked ~29 ms. **Timestamp beats=2:** `desired2present` **5 ms=7072**; FPS 125.062; 25 ms delta 0. | Default timestamp on. Skip probe off. No Mailbox. |
| N6 | Second adapter/device from `preferred_canvas_format` → `resolve_device(0)` | **Closed** | Reuses the guest device (2026-09-01). Was a boot bug, not a 5 s period. | — |
| N8 | `webgpu.h` main vs Dawn `.so` SHA ABI mismatch | **Closed** (this hitch) | Adapter/device/present run; cube is on-screen. | Only reopen if present starts returning Error. |
| N9 | Dawn C `wgpuSurfacePresent` vs androidx `GPUSurface.present` submission details | **Closed** (2026-09-01, non-divergence) | `surface-caps formats=[22,23,40,30] present=[4,1] alpha=[4]` → RGBA8 first, `Fifo(1)`+`Mailbox(4)`, `Inherit(4)`. `AlphaMode_Auto` native-defaults to `Inherit`, so aligning `configure` to `caps.alphaModes[0]` is a no-op. `PRESENT_FIFO=0x0001` correct. Dawn pin = androidx AAR SHA (same commit). 65 s / 7835 frames: `droppedFrames=0`, `jankyFrames=0`, `present2presentDelta 25ms=0`, `desired2present 5ms=7811`. | Only remaining diffs: D24's real `onSubmittedWorkDone` fence vs NativeGpu mark+retire; NativeGpu auto-presents in `queue.submit` (H8). |

## 3. Suggested kill order (one variable)

Do **not** restack DisplayManager / GameState / SurfaceControl votes. Do **not** treat another JNI removal as the next hitch cut.

1. **N4 + D2** — **Done** (2026-09-01 ~25 s): acquire `>20ms=0`; `vsync→present` `>8.3ms=0` (JNI path was ~19%). Stall Closed. Cross-beat phase Closed on this sample.  
2. **D3 / D19 / N5** — **Done** timestats (2026-09-01). BLAST **5**. Timeline class n/a; `desired2present` bimodal = extra queued vsync.  
3. **Skip-present `n=6` A/B** — **Done**: extra-vsync cluster drained; 25 ms rewind deltas gone. Property default **off** (not a product 100 fps). No Mailbox.  
4. **D21** — **Done** (2026-09-01 ~22 s): display FrameTimeline 2155/2155 On-time + Valid prediction; predictor recovery off; SurfaceView layer timeline n/a.  
5. **D22** — **Done** (2026-09-01 Settings lock, no app vote): 25 ms rewind class gone this ~60 s; stuffing parked at ~29 ms.  
6. **D3 timestamp** — **Done** (2026-09-01): `ANativeWindow_setBuffersTimestamp` default 2 beats; `desired2present` 29 ms → 5 ms at 125 fps.  
7. **N2** — only if Dawn C starts UAF / SIGSEGV on recycle (crash lane).  
8. **D13 / keep vs BLAST** — **Closed (worsens):** keep-6 raised hitch frequency; BLAST stayed 5. Reverted to keep-3. Do not restack keep.

## 4. This path already kept

- H1 / C2 / H8 / Fifo / intern one queue / keep-3 (ND-SURF; keep-6 reverted).  
- No C7 AAR leak batching.  
- `preferred_canvas_format` reuses the guest device (N6).  
- Out-of-tree host: peak mode + same-Hz `setFrameRate` (H24 / H27). Do not add more votes.

## 5. Bottleneck after D25 (hitch-branch probes)

Not a millisecond hotspot. Hitch-branch JNI-path work was **~7.1 ms / 8.33 ms beat**. Two stages:

1. **D2 (mid-chain) — phase source.** CM pump serializes guest into the present deadline. `wakeA` condvar is stable (0.5–1.3 ms). The 15 androidx JNI imports (~3.3 ms) are **not necessary** for this pop (D25). **Dawn C 2026-09-01:** `vsync→wgpuSurfacePresent` `>8.3ms=0` over 2880 presents (JNI path ~19%). Latency still wanders 0.8–6.9 ms *inside* the beat. Crossing the vsync boundary is not required for the remaining hitch. **P2/P3 (95 s / 12000 frames, 2026-09-01):** present `phase-crossing` 2 (55.5 s apart, ultra-slow drift); retire age `<8.3ms=0`. **P4 (95 s / 12720 frames, 2026-09-01):** guest angle clock `angleDt 8-9ms=all`, `9-17ms=0`, `>17ms=0` — the transform matrix is **strictly linear in time**. **P5 (~12 min, 2026-09-02):** `take-skip` **0** — the guest `now`/`angle` steps exactly one 8.33 ms beat, **decoupled from present phase**. **Phase addendum (2026-09-02):** re-reading the full `phase-crossing` sequence found present phase can **snap out of lock** — once (23:44), `lat` 5→9.6 ms, `margin` +3→−1.3 ms in one beat, then `cross` several/s; but it was seen **once in 30 min**, and P5 shows the snap does **not** propagate into guest content. CM pump / `GfxOnFrameGate` phase lock, host present/buffer lifetime, and guest content are all **clean** — the out-frame path is not the hitch.
2. **D3/D19 (end of chain) — where the eye sees it.** Untimestamped `present` + **5** BLAST stuffed the queue (~29 ms `desired2present` under D22 lock). **Timestamp beats=2** (2026-09-01): `ANativeWindow_setBuffersTimestamp` before `wgpuSurfacePresent`; `desired2present` **5 ms=7072**, FPS 125. Display FrameTimeline stays On-time. D13 is the leftover visual symptom if any. **D20 Closed (2026-09-01):** no SF-side throttle. **N9 (2026-09-01):** the present submission path itself is clean — configure params (format `caps.formats[0]`, `Fifo`, `alpha=caps.alphaModes[0]`), Dawn commit (same androidx AAR SHA), and a 65 s SF window (`droppedFrames=0`, `jankyFrames=0`, `present2presentDelta 25ms=0`, `desired2present 5ms=7811`) show **no rewind/drop/jank** on the Dawn C path.

**Conclusion (2026-09-02, verification paused):** all three observable layers — guest content (P4/P5 linear, decoupled), SF counters (`droppedFrames=0`, `jankyFrames=0`, no rewind/drop), and present submission (N9 clean) — are **clean**. The visual pop therefore sits in an un-measured layer: most likely the compositor/panel BLAST re-show (D13/D19) outside SF's counters, or the rare present-phase snap's host-side transient — neither yet frame-captured at the exact pop instant. Next needs event-triggered capture (screenrecord keyed on `sinceLast` collapse to pull the frames around the pop) or the real `onSubmittedWorkDone` fence lane to close the last D24 structural gap.

Fix ranking: present timestamp **landed**. Draw-present split is leftover. Do not recut JNI. Cube host for this probe: out-of-tree `hosts/fullscreen-surface` + `GpuBackends.dawn()` + `Store.bindCanvasNativeWindow`.
