# Gap: `wasi:webgpu` WIT ↔ host ↔ androidx.webgpu

**English** | [中文](gap-webgpu-wit-androidx.zh.md)

Living map for the **JNI / androidx leftover** (`GpuBackends.dawnJni()`, `id = "dawn-jni"`). Pin: `wasi:webgpu@0.3.0-rc.2` ([`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit)). Dawn AAR: `androidx.webgpu:webgpu:1.0.0-alpha05`. Do not treat this page as a cut queue. P0 close-out: [`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md). **Product default** is NativeGpu / Dawn C: [`gap-webgpu-native-dawn.md`](gap-webgpu-native-dawn.md). Playbook: [`../agent/native-dawn.md`](../agent/native-dawn.md). Use this table as the **mapping spec** when translating `DawnWasiWebGpuHost`. Product claim (not CTS): [`../scheme/claim-010.md`](../scheme/claim-010.md).

**Degree**

| Tag | Meaning |
|-----|---------|
| **Dawn** | Guest value reaches `androidx.webgpu` / Dawn |
| **Record** | JNI packed the WIT field onto a Kotlin `WasiWebGpuHost` record; Dawn ctor has no slot |
| **Shape** | Guest import matches WIT; L2 described JNI |
| **Fixture** | Test-only constructor (`get-gpu`, `get-device`, …) — not product WIT |
| **Flat** | Frozen experimental / dual-register `u32` name — do not extend |
| **Out** | Explicit non-goal (CTS, wasi-gfx, rewrite Dawn) |

## 1. Coverage

| Axis | Degree |
|------|--------|
| `[method]` names vs pin | **Shape** — all 224 pin resource methods registered in `native/src/cm.rs`. Product consume degree is Dawn C ([`../scheme/claim-010.md`](../scheme/claim-010.md)) |
| S1–S5 (`queue`, `request-adapter` `option`, `request-device` `result`, `create-buffer`, `submit` list) | **Shape** + **Dawn** on the `dawn-jni` leftover path |
| Leftover optional descriptors (F1–F9) | **Shape** into Kotlin records; Dawn copy where the AAR allows |
| G1–G9 consume + WG-6 guest-drawn compute / 3D / canvas present | **Dawn** (see holes below) |
| Unwired store | `gpu.request-adapter` → guest **`none`** (not a trap) |

## 2. androidx holes (Record, not Dawn)

These are the only P0 fields still dropped on the GPU. Kotlin records keep the guest values. Revisit only when bumping `androidx.webgpu` (changelog the AAR pin). Do **not** re-cut G3 / G8.

| WIT | Kotlin | androidx `1.0.0-alpha05` |
|-----|--------|-------------------------|
| `gpu-shader-module-descriptor.compilation-hints` | `ShaderModuleDescriptor.compilationHints` | `GPUShaderModuleDescriptor` is label + WGSL/SPIR-V only |
| `gpu-canvas-configuration.color-space` | leftover on canvas configure (`colorSpace`, `-1` = absent) | `GPUSurfaceConfiguration` has no color-space slot |
| `gpu-canvas-configuration.tone-mapping` | leftover (`toneMapping`, `-1` = absent) | no tone-mapping slot |

Related slots that **do** exist on this AAR (already consumed): primitive cull/front/strip, blend, multisample, texture `viewFormats`, `requestAdapterWebXROptions`, `GPUDeviceDescriptor.defaultQueue`, color `writeMask`, depth-stencil faces/masks/bias, surface `viewFormats` / `alphaMode`, compute `layout` omitted = auto.

## 3. Host vs WIT (by family)

Guest boundary is the pin. Dawn handles stay `u32` **reps** behind `own`/`borrow`.

| Family | WIT | Host / Dawn | Degree |
|--------|-----|-------------|--------|
| Adapter / device / queue | async `option`/`result`; `required-features` list; `required-limits` map including stage storage keys | Dawn request + `GPULimits` / compatibility-mode setters | **Dawn** |
| Buffer / texture / sampler / views | descriptors + map-async true async | described JNI → Dawn | **Dawn** |
| Shader module | WGSL + label + hints | hints stay on record | **Dawn** code, **Record** hints |
| Pipelines | vertex buffers, blend, MSAA, write-mask, stencil leftovers, constants, `layout: auto` | Dawn ctors; auto = omit layout | **Dawn** |
| Passes / copies / debug / bundles / query-sets | recording commands | described JNI → Dawn/Cpu | **Shape** + **Dawn** |
| Canvas | `configure` device+format+usage+leftovers; `get-current-texture`; guest-drawn present | `GPUSurfaceConfiguration`; present is host `exp_surface_*` internally, guest names stay `gpu-canvas-context.*` | **Dawn** except color-space / tone-mapping **Record** |
| Labels / info getters / supported-limits | WIT getters | described JNI | **Shape** + **Dawn** |
| `get-*` test constructors | not in product WIT | fixtures / instruments | **Fixture** |
| `experimental:webgpu-cm` flats (`device-get-queue`, `queue-submit1`, …) | not pin product names | still registered | **Flat** |

Cpu host (`CpuWasiWebGpuHost`) is a stand-in: VectorAdd shader-text match only. Product GPU path is NativeGpu / Dawn C; this page is the `dawn-jni` leftover.

## 4. Out of scope

| Item | Tag |
|------|-----|
| WebGPU CTS / “compliant wasi:webgpu” | **Out** (NG-5) |
| Product `surface-*` / wasi-gfx as a **P0** re-queue | **Out** (NG-9). Minimal present loop is **`0.1.0`**: [`../scheme/rfc-wasi-gfx-frame-loop.md`](../scheme/rfc-wasi-gfx-frame-loop.md) |
| Second Dawn renderer | **Out** (NG-7) |
| `wasmtime-wasi` as the WebGPU host | **Out** — GPU is `:host-dawn` |

## 5. Android facts (not a cut queue)

Recorded on Vivo V2458A (Android 16, `arm64-v8a`, Mali-G925-Immortalis MC12) with the out-of-tree rotating-cube demo (continuous `on-frame`, not the 500ms GFXV instrument).

| Observation | Host fact |
|-------------|-----------|
| Cube rotation **hitches**, hitch rate **rises**, then GpuThread **SIGSEGV** (`fault addr 0x20`) in `nativeCallRunConcurrent` after ~10–13s | Product `gpu-canvas-context.get-current-texture` inserted a new `HandleTable` `GPUTexture` every frame. `queue.submit` / `context.present` presented but **did not `tryDrop`/close** that texture (Track A `surfaceGetCurrentTextureView` already recycled View↔Texture). Dawn never returned the BLAST image; Mali stalled then crashed. Closing the image in the same `present()` or on the next acquire UAFd Mali (`0x20` / `0x1f8`). A CPU-frame keep-last-N ring without a GPU fence still crashed (~45s) once hitching let the CPU run ahead of BLAST. Blocking GpuThread on the **current** `onSubmittedWorkDone` stacked on vsync and dropped beats. Recycle: `tryDrop` after GPU done and **3** newer frames, retired on the event poller (not on vsync→present). Guest-owned textures are not swept. |
| Cube spins **faster just after launch**, then hitches | Pin `frame-event` is `{ nothing: bool }` (no rAF timestamp). Guest used `angle += const` per beat; V2458A Choreographer is 120 Hz. A host ~60 Hz cap hid the fast start but left hitching (every-other-vsync jitter). Do not cap `on-frame`. Guest delta is `wasi:clocks/monotonic-clock#now` (same role as rAF `t - last`). |
| `CompositeAlphaMode::Opaque` rejected for this window | Guest should leave canvas `alpha-mode` unset (host picks a capability). Not an androidx hole in §2. |
| `create-texture` `depth24plus` observed as `RGBA8Unorm` on this path | Guest may skip depth; mapping hole vs pin, not a P0 re-cut. |
| GFXV instrument did not catch this | `CLOSE_AFTER_VSYNC_MS = 500`. Leak needs seconds of present. Cpu recycle: `WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest`. |
| Cube **still hitches** after recycle / no 60 Hz cap / clocks dt / in-frame vsync drop | Host beat is 1:1 ([`gfx-hitch-checklist.md`](gfx-hitch-checklist.md)). Guest-side trig lives in the out-of-tree examples repo. |

When androidx grows a hole’s ctor argument: copy the existing Kotlin field into Dawn in that pin-bump PR, and update this table. Do not reopen G1–G9 or F1–F9 as queues.
