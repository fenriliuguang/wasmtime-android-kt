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

**Filter.** Bypassing ART / Kotlin / androidx JNI on the GPU hot path **does not** remove the pop. The native-dawn “Why” (phase jitter amplified by frequent ART crossings) is **Closed for this hitch**. What is still shared is: guest + `GfxOnFrameGate` / `postGfxVsync` + 8MiB `wasmtime-cm-pump` + hitch ring (keep-3 / Fifo / H8) + Dawn `present()` with **no** presentation timestamp + SurfaceView BLAST + OEM VRR.

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
| **D2** | Vsync → `postGfxVsync` → 8MiB CM pump → Dawn present **phase** (no `lastDtNs` spike) | Closed as *stall*; playbook still names phase | **Open** / **Likely** | Shared. A/B killed the “ART crossings” amplifier story. Measure vsync→`wgpuSurfacePresent` ns. |
| **D3** / **D19** | `present()` has no presentation timestamp; BLAST stuffing; SF re-shows *n−2* | Open / Likely | **Open** / **Likely** | Shared. `timestats` `presentToPresent`; Perfetto `BUFFER_STUFFING`. Skip-present only if stuffing is proven. No Mailbox. |
| **D13** | Cube *looks* like it rewinds while `angle` already advanced | Likely (symptom) | **Likely** | Distinguishes guest dt from D3. |
| **D20** | SF hold-until-phase + UID FPS divisor | Open | **Open** | Shared. `dumpsys SurfaceFlinger` layer override. Do **not** GameState. |
| **D21** | VSyncPredictor periodic correction | Open / Trace | **Open** / **Trace** | Shared. Perfetto around a pop. |
| **D22** / **H27** | Vivo RMS / smart refresh; app votes worsen it | Open / Likely; app votes Closed | **Open** / **Likely** | Settings lock 120 Hz. Do not restack DisplayManager votes. |
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
| N2 | `mark_canvas_gpu_done()` on `queue.submit` is immediate; no `wgpuQueueOnSubmittedWorkDone` | **Open** (lifetime) / **Closed** (5 s period) | `native_gpu.rs` marks the whole ring `gpu_done` then keep-3 retires. androidx waited a real fence on the poller. | Wire C-API work-done if textures UAF. Not a 5 s timer. |
| N3 | `wgpuInstanceProcessEvents` on acquire (pump thread) vs androidx poller | **Open** / **Trace** | Same thread as present. Can add phase; should not spike Choreographer. | Log acquire ns + `GetCurrentTexture` status (was `GfxHitch` on JNI). |
| N4 | No Dawn C acquire / present-interval histogram | **Trace** | androidx had `GfxHitch`. Dawn C logcat is boot-only (`dlopen` / adapter / device). | Add the same `<11 / 11–20 / >20ms` buckets on NativeGpu. |
| N5 | `wgpuSurfacePresent` vs androidx `GPUSurface.present` stuffing | **Open** | Both go `ANativeWindow` → BLAST. gfxinfo count not re-taken on Dawn C. | `dumpsys gfxinfo` BLAST size; compare to 5. |
| N6 | Second adapter/device from `preferred_canvas_format` → `resolve_device(0)` | **Closed** | Reuses the guest device (2026-09-01). Was a boot bug, not a 5 s period. | — |
| N8 | `webgpu.h` main vs Dawn `.so` SHA ABI mismatch | **Closed** (this hitch) | Adapter/device/present run; cube is on-screen. | Only reopen if present starts returning Error. |

## 3. Suggested kill order (one variable)

Do **not** restack DisplayManager / GameState / SurfaceControl votes. Do **not** treat another JNI removal as the next hitch cut.

1. **N4** — NativeGpu acquire + present-interval histogram. If `>20ms=0` through a pop (same as androidx), stall-type stays Closed.  
2. **D3 / D19 / N5** — `dumpsys SurfaceFlinger --timestats` + `gfxinfo` BLAST on the Dawn C SurfaceView.  
3. **D2** — vsync `frameTimeNanos` → `wgpuSurfacePresent` return (pump thread). Phase without a histogram hole.  
4. **D21** — Perfetto FrameTimeline ~20 s around a pop (`BUFFER_STUFFING` / `PREDICTION_ERROR`).  
5. **D22** — Settings lock 120 Hz (no app vote).  
6. **Skip-present every N** — only if timestats shows stuffing. No Mailbox.  
7. **N2** — only if Dawn C starts UAF / SIGSEGV on recycle (crash lane, not this pop).

D24 control stays the no-Wasmtime androidx cube. A no-Wasmtime **Dawn C** cube is a later A/B if D2/D3 stay stuck; do not vendor it.

## 4. This path already kept

- H1 / C2 / H8 / Fifo / intern one queue / keep-3 (ND-SURF invariants).  
- No C7 AAR leak batching.  
- `preferred_canvas_format` reuses the guest device (N6).  
- Out-of-tree host: peak mode + same-Hz `setFrameRate` (H24 / H27). Do not add more votes.
