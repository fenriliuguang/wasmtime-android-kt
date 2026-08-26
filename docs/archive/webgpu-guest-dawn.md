# Agent playbook: Dawn consume + WG-6 leftovers

> **Archived 2026-08-22.** Do not implement from this file. P0 close-out: [`p0-wasi-webgpu.md`](p0-wasi-webgpu.md). Gap: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md). Current queue: [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md).

**English** | [中文](webgpu-guest-dawn.zh.md)

Use this **after** guest-pipeline P1–P5 and leftover-descriptor semantics F1–F9 (including named handle-0 and full `required-features`). Closed: [`webgpu-guest-pipeline.md`](webgpu-guest-pipeline.md), [`webgpu-guest-semantics.md`](webgpu-guest-semantics.md). Do **not** re-hang `[method]` names, re-cut P1–P5 / F1–F9, re-cut labels / limits / `create-sampler` first-cut, re-cut canvas first-cut (`device`+`format`+`usage`), or re-cut Dawn compute/render **cite** slices.

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

RFCs (do not rediscover policy mid-cut): [`guest-shape.md`](../scheme/guest-shape.md), [`rfc-ecosystem-contribution.md`](../scheme/rfc-ecosystem-contribution.md), [`rfc-pluggable-gpu-backend.md`](../scheme/rfc-pluggable-gpu-backend.md), [`roadmap-wasi-webgpu.md`](../scheme/roadmap-wasi-webgpu.md). **One lane, one PR.**

## Why this queue

JNI already packed leftover optional fields into Kotlin host records (F1–F9). Dawn `androidx.webgpu` still drops several of those fields, a few WIT fields were never snapshotted, `layout: auto` traps, and WG-6 still lacks a **guest-drawn** compute/3D/present slice.

Empty compute `submit`, 1×1 color-clear cite, `@builtin(vertex_index)` offscreen triangles, and host-cleared canvas present are **not** this queue’s DoD. Test-only `get-gpu` / `get-device` / `get-canvas-context` stay fixtures — do not replace them.

| Guest wants | Blocker today |
|-------------|----------------|
| Blend / cull / MSAA **on GPU** | Kotlin `RenderPipelineDescriptor` has blend / primitive extras / `multisample`; Dawn `GPUPrimitiveState(topology=…)`, `GPUColorTargetState(format=…)`, no `multisample` |
| Texture `view-formats` **on GPU** | Kotlin `TextureDescriptor.viewFormats`; Dawn `GPUTextureDescriptor` omits them |
| Shader compilation-hints **on GPU** | Kotlin `ShaderModuleDescriptor.compilationHints`; Dawn `GPUShaderModuleDescriptor` is label + WGSL |
| `xr-compatible` **on GPU** | Kotlin `RequestAdapterOptions.xrCompatible`; Dawn `GPURequestAdapterOptions` is power / fallback / Vulkan |
| `default-queue` label **on GPU** | Kotlin `DeviceDescriptor.defaultQueueLabel`; Dawn `GPUDeviceDescriptor` has no `defaultQueue` |
| Color `write-mask` | WIT `gpu-color-target-state.write-mask`; Kotlin `ColorTargetState` is format + blend only |
| Depth stencil / bias | WIT stencil faces, masks, depth-bias; Kotlin `DepthStencilState` is format / write / compare |
| Full canvas configure | JNI `canvasContextConfigureDescribed` is `(IIII)I` (device+format+usage); WIT also has view-formats / color-space / tone-mapping / alpha-mode |
| Pipeline `layout: auto` | Dawn/Cpu `Unsupported("auto pipeline layout; pass an explicit pipeline-layout handle")` |
| WG-6 real slice | Cite is empty compute / 1×1 clear; canvas present is host clear + present, not a guest-drawn frame |

## Select the cut

If the user named a lane or `[method]` list, keep **one** family. Otherwise:

```powershell
.\scripts\webgpu-guest-dawn-remaining.ps1
```

No `pwsh`: `python3 ./scripts/webgpu-guest-dawn-remaining.py` (same flags: `--all`).

Do the printed **Next:** line. Do not Grep `cm.rs` for a queue. Default order: **G1 → G9**. WG-6 real slices and stage-only required-limits keys never auto-select.

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu. Grep the vendored WIT.
- Do **not** create, reopen, or request GitHub Issues (or Discussions-as-issues) on any upstream, including [wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) and Wasmtime. No `gh issue create`. Record Android facts in this repo only.
- Do **not** read `native/src/cm.rs`, `native/src/jvm.rs`, `native/src/webgpu_abi.rs`, `ExperimentalHostCallbacks.kt`, `ExperimentalWebGpuBridge.kt`, `HostWebGpuBackend.kt`, `WasiWebGpuHost.kt`, Cpu/Dawn hosts, or RFCs without an offset. Grep the symbol, then Read ~80 lines.
- Do **not** open a third native/Kotlin file to discover a template. Copy the **one** stack below.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** run full `cargo test --tests` or device instruments unless this PR **is** a named WG-6 lane.
- Do **not** register experimental `surface-*` as product WIT. Host may call existing `exp_surface_*` from Kotlin; guest names stay `gpu-canvas-context.*`. No wasi-gfx (NG-9). No CTS / compliance claim (NG-5).
- Do **not** add new `HostArg` variants. Pack with existing `Int` / `Long` / `Str` / `Ints` / `Longs` / `Bytes` / `Float`. Indexed add-then-create JNI (like `record-*`) is OK — still one WIT wrap.
- Do **not** rewrite `fixtures/w1/README.md` **Transitional:**. Table row + two `wasm-tools` lines only.
- Do **not** re-cut P1–P5, F1–F9, sampler/view leftovers, pipeline-constant **map resource**, S1–S3 JNI, canvas first-cut, or Dawn compute/render cite.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

`request-adapter` with no backend still returns guest **`none`**. Keep true `async` wraps (`func_wrap_concurrent` + yield) for WIT `async func`.

## Lanes (auto)

G1–G5 sentinels live in `DawnWasiWebGpuHost.kt` (remaining drops a lane only when that block **changes**). G6–G7 sentinels live in `HostTypes.kt`. G8 is the `canvasContextConfigureDescribed` JNI signature. G9 is the auto-layout throw string. Forward into existing `WasiWebGpuHost` records; extend those records when a field is missing. Do not invent a second GPU stack (NG-7). If `androidx.webgpu` has no matching constructor argument, still populate the Kotlin record, note the androidx hole in the changelog fragment, and change the remaining sentinel (comment or dummy field) so the lane does not auto-repeat.

| PR | Method | DoD |
|----|--------|-----|
| G1 | Dawn consume render-pipeline extras | Copy Kotlin `blend` / primitive **cull-mode** + **front-face** + **strip-index-format** / `multisample` into `GPUColorTargetState` / `GPUPrimitiveState` / `GPURenderPipelineDescriptor`. Remaining sentinel: the F1 topology-only + format-only `GPURenderPipelineDescriptor` block in `DawnWasiWebGpuHost.kt`. Do **not** re-cut F1 JNI |
| G2 | Dawn consume texture view-formats | `GPUTextureDescriptor.viewFormats` from `TextureDescriptor.viewFormats` (empty → none / empty array). Remaining sentinel: the no-`viewFormats` `GPUTextureDescriptor(` block. Do **not** re-cut F3 JNI |
| G3 | Dawn consume shader compilation-hints | `GPUShaderModuleDescriptor` takes snapshotted `compilationHints` when androidx has a slot. Remaining sentinel: label + WGSL-only `GPUShaderModuleDescriptor(`. Do **not** re-cut F5 JNI |
| G4 | Dawn consume `xr-compatible` | `GPURequestAdapterOptions` takes `xrCompatible` when androidx has a slot. Keep power / fallback / **Vulkan** `backendType`. Remaining sentinel: the options block with no `xrCompatible`. Do **not** re-cut F6 JNI |
| G5 | Dawn consume `default-queue` | `GPUDeviceDescriptor.defaultQueue` (or equivalent) from `defaultQueueLabel` when androidx has a slot. Keep `requiredFeatures` / `requiredLimits` / callbacks. Remaining sentinel: `requiredLimits = …` immediately followed by `deviceLostCallbackExecutor`. Do **not** re-cut F7 JNI |
| G6 | `gpu-device.create-render-pipeline` write-mask | Guest per-target `write-mask` (absent → none / all) on `ColorTargetState` and into Dawn when the slot exists. Change `deviceCreateRenderPipelineDescribed` JNI **or** `ColorTargetState` so remaining drops G6. Fixture: at least one non-default mask **or** explicit `all` reaches the host record |
| G7 | `gpu-device.create-render-pipeline` depth-stencil leftovers | Guest stencil-front/back, stencil masks, and depth-bias fields when present. Extend `DepthStencilState`; copy into Dawn `GPUDepthStencilState` when androidx has slots. Remaining sentinel: format/write/compare-only `DepthStencilState`. Depth attachment on begin-render-pass stays P4 — do not re-cut it |
| G8 | `gpu-canvas-context.configure` leftovers | Guest `view-formats` / `color-space` / `tone-mapping` / `alpha-mode` (absent → none) on top of existing device+format+usage. Change `canvasContextConfigureDescribed` JNI signature. Do **not** re-cut canvas first-cut or present cite |
| G9 | pipeline `layout: auto` | `create-compute-pipeline` (and render if it still rejects auto) must not throw `auto pipeline layout; pass an explicit pipeline-layout handle` when the guest uses auto. Prefer Dawn auto layout if androidx exposes it; otherwise changelog the hole and change the throw sentinel. Explicit pipeline-layout handles stay valid |

Copy: G1–G5 / G9 deepen **existing** Dawn mapping (`deviceCreateRenderPipeline` / `deviceCreateTexture` / …). G6–G8 copy `[method]gpu-device.create-buffer` + `jvm::exp_*_described` + `attach*` / `ForwardingHostCallbacks`. Do not add a second wrap.

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| WG-6 real guest compute | User named 真 compute / bind-group dispatch / WG-6 compute | Guest chain on `DawnWasiWebGpuHost`: BGL + bind-group + compute pipeline + `set-bind-group` + `dispatch-workgroups` + `queue.submit`. **Not** empty `begin-compute-pass`. **Not** Cpu `VectorAddScenario` shader-text match. One instrument. No CTS |
| WG-6 real guest 3D | User named 真 3D / vertex draw / WG-6 render | Guest chain on Dawn: vertex buffers + `draw` or `draw-indexed` + `queue.submit`. **Not** the 1×1 color-clear cite. Depth optional this cut. One instrument. No CTS |
| WG-6 canvas present guest-drawn frame | User said 上屏 / canvas / present **and** guest-drawn | Render target is `gpu-canvas-context.get-current-texture`; **guest** records the pass; host presents that frame (not host-only clear). **No** product `surface-*` names. Test-only `get-canvas-context` may stay. Do **not** re-cut canvas first-cut |
| Stage-only required-limits keys | User named stage-only storage limits | Map `max-storage-*-in-vertex-stage` / `fragment-stage` if androidx grows setters. Do **not** re-cut F9 `record-option-gpu-size64` |

## File whitelist

- `native/src/cm.rs` — this method’s wrap only (windowed); G1–G5 / G9 usually skip
- `native/src/jvm.rs` — described JNI (G6–G8: signature or packing **must** change so remaining drops the lane)
- `runtime-api/.../ExperimentalHostCallbacks.kt`
- `host-dawn/.../HostWebGpuBackend.kt` (`ForwardingHostCallbacks` — product path)
- `host-dawn/.../ExperimentalWebGpuBridge.kt` — existing attach for this method
- `host-dawn/.../HostTypes.kt` — add missing record fields for this family only
- `host-dawn/.../DawnWasiWebGpuHost.kt` — G1–G5 / G9, or when this family’s record already exists and Dawn still drops it
- `host-dawn/.../CpuWasiWebGpuHost.kt` — **G9 only** (same auto-layout throw)
- `host-dawn/.../AbiCmHostBindings.kt` **only** if `WasiWebGpuHost` cannot already take the Kotlin record
- `fixtures/w1/webgpu_method_<slug>.{wat,wasm}` — guest values; re-parse/validate `cm-async,component-model`
- `native/tests/wasi_webgpu_method/<slug>.rs` — **assert lifted fields**; duplicate WIT types locally (no `use crate::webgpu_abi`)
- `smoke-app/.../WasiWebGpuMethod*InstrumentedTest.kt` — named WG-6 lanes only, and only if that attach already exists
- `changelog/unreleased/<yyyy-mm-dd>-dawn-<slug>.md` (three bullets)
- `fixtures/w1/README.md` — those table rows

Do not add files under `docs/archive/`. Tests **must not** `use crate::webgpu_abi`. Register `resource()` **before** wraps that use that `Resource<T>`. Use guest `rep` when `!= 0`; do not rebuild adapter → device → encoder when the handle is live.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module>
```

Root `bash ./gradlew :runtime-api:compileKotlin` if Kotlin callbacks changed. `:host-dawn:compileDebugKotlin` when Android SDK is present (this Cloud image often has none — do not fail the lane solely on that). Named WG-6: that instrument family only.

PR title: `feat(webgpu): L2 dawn consume <family>` (G6–G8: `feat(webgpu): L2 <resource> <family> guest fields to host`; G9: `feat(webgpu): L2 auto pipeline layout`; WG-6: `feat(webgpu): WG-6 <compute|render|canvas present guest-drawn>`). Label `enhancement`.

User prompt that works: “follow `docs/agent/webgpu-guest-dawn.md`” or name the lane (`G1 blend/cull on GPU`, `G8 canvas configuration`, `WG-6 real compute`).
