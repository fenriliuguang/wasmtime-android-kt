# Agent playbook: leftover descriptor semantics

**English** | [中文](webgpu-guest-semantics.zh.md)

Use this **after** guest-pipeline P1–P5, sampler/view leftovers, pipeline-constant **JNI reps**, S1–S3 leftover descriptor JNI, canvas present, and Dawn render cite. Closed: [`webgpu-guest-pipeline.md`](webgpu-guest-pipeline.md). Do **not** re-hang `[method]` names, re-cut labels / limits / `create-sampler` first-cut, re-cut canvas first-cut, or re-cut those JNI first-cuts.

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

RFCs (do not rediscover policy mid-cut): [`guest-shape.md`](../scheme/guest-shape.md), [`rfc-ecosystem-contribution.md`](../scheme/rfc-ecosystem-contribution.md), [`rfc-pluggable-gpu-backend.md`](../scheme/rfc-pluggable-gpu-backend.md), [`roadmap-wasi-webgpu.md`](../scheme/roadmap-wasi-webgpu.md). **One lane, one PR.**

## Why this queue

Shape is hung and P1–P5 lists/layouts/depth/mip are on the host records. Real guests still lose **optional descriptor fields** that JNI never packed, or that Dawn never copies from the already-snapshotted Kotlin record into `androidx.webgpu`.

Empty compute `submit` and `@builtin(vertex_index)` offscreen triangles are **not** this queue’s DoD. Test-only `get-gpu` / `get-device` / `get-canvas-context` stay fixtures — do not replace them.

| Guest wants | Blocker today |
|-------------|----------------|
| Blend / MSAA / cull / strip | `create-render-pipeline` has vertex buffers + target format; `PrimitiveState` is topology only; `ColorTargetState` is format only; Dawn `GPUPrimitiveState(topology=…)` / `GPUColorTargetState(format=…)` |
| MRT | `begin-render-pass` JNI is **first** color + depth; `RenderPassDescriptor.colorAttachments` already a list — Dawn maps the list |
| Texture view-formats | `create-texture` size/format/usage/mip/sample/dimension; Kotlin `label` exists; no `viewFormats`; Dawn `GPUTextureDescriptor` omits them |
| Mapped buffer + labels | Dawn `GPUBufferDescriptor` already takes `mappedAtCreation` / `label`; JNI `deviceCreateBufferDescribed` is `(device, size, usage)` only |
| Shader hints + label | Dawn takes `label` + WGSL; JNI is `(device, code)` only; no `compilation-hints` |
| XR / default queue | `request-adapter` has power / fallback / `feature-level`; no `xr-compatible`. `request-device` has first feature / required-limits rep / label; no `default-queue` |
| Pipeline constants **on GPU** | JNI + host snapshot already fill `ProgrammableStage.constants`; Dawn `GPUComputeState` / vertex / fragment omit the map |
| Required-limits **on GPU** | JNI snapshot fills `DeviceDescriptor.requiredLimits`; Dawn `GPUDeviceDescriptor` is label + callbacks only |

## Select the cut

If the user named a lane or `[method]` list, keep **one** family. Otherwise:

```powershell
.\scripts\webgpu-guest-semantics-remaining.ps1
```

No `pwsh`: `python3 ./scripts/webgpu-guest-semantics-remaining.py` (same flags: `--all`).

Do the printed **Next:** line. Do not Grep `cm.rs` for a queue. Default order: **F1 → F9**. SupportedLimits handle-0 never auto-selects.

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu. Grep the vendored WIT.
- Do **not** create, reopen, or request GitHub Issues (or Discussions-as-issues) on any upstream, including [wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) and Wasmtime. No `gh issue create`. Record Android facts in this repo only.
- Do **not** read `native/src/cm.rs`, `native/src/jvm.rs`, `native/src/webgpu_abi.rs`, `ExperimentalHostCallbacks.kt`, `ExperimentalWebGpuBridge.kt`, `HostWebGpuBackend.kt`, `WasiWebGpuHost.kt`, Cpu/Dawn hosts, or RFCs without an offset. Grep the symbol, then Read ~80 lines.
- Do **not** open a third native/Kotlin file to discover a template. Copy the **one** stack below.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** run full `cargo test --tests` or device instruments unless this PR **is** the named handle-0 lane.
- Do **not** register experimental `surface-*` as product WIT. No wasi-gfx (NG-9). No CTS / compliance claim (NG-5).
- Do **not** add new `HostArg` variants. Pack with existing `Int` / `Long` / `Str` / `Ints` / `Longs` / `Bytes` / `Float`. Indexed add-then-create JNI (like `record-*`) is OK — still one WIT wrap.
- Do **not** rewrite `fixtures/w1/README.md` **Transitional:**. Table row + two `wasm-tools` lines only.
- Do **not** re-cut P1–P5, sampler/view leftovers, pipeline-constant **map resource**, S1–S3 JNI (`feature-level` / required-limits rep / label), or canvas present.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

`request-adapter` with no backend still returns guest **`none`**. Keep true `async` wraps (`func_wrap_concurrent` + yield) for WIT `async func`.

## Lanes (auto)

JNI sentinels live in `native/src/jvm.rs` (F1–F7). F8–F9 sentinels live in `DawnWasiWebGpuHost.kt`. Remaining drops a lane only when that sentinel **changes**. Forward into existing `WasiWebGpuHost` records; extend those records when a field is missing. Do not invent a second GPU stack (NG-7). If `androidx.webgpu` has no matching constructor argument, still populate the Kotlin record, note the androidx hole in the changelog fragment, and change the remaining sentinel (comment or dummy field) so the lane does not auto-repeat.

| PR | Method | DoD |
|----|--------|-----|
| F1 | `gpu-device.create-render-pipeline` | Guest **blend** (per color target, absent → none) + **multisample** (`count` / mask / alpha-to-coverage when present) + primitive **cull-mode** + **strip-index-format** (and front-face if already on the WIT record). Topology may stay TriangleList if unset. Fixture: at least one of blend, sample-count≠1, or cull ≠ none reaches the host record. Change the `deviceCreateRenderPipelineDescribed` JNI signature |
| F2 | `gpu-command-encoder.begin-render-pass` | **All** color attachments (view + load/store + optional clear), not `listOf(first)`. Depth-stencil already described — keep it. Empty extra list stays valid. Fixture: ≥2 color views. Change `beginRenderPassDescribed` JNI signature |
| F3 | `gpu-device.create-texture` | Guest `view-formats` (empty → none) + `label` (empty → none) on top of existing size/format/usage/mip/sample/dimension. Change `deviceCreateTextureDescribed` JNI signature |
| F4 | `gpu-device.create-buffer` | Guest `mapped-at-creation` + `label` into existing `BufferDescriptor` (Dawn already consumes both). JNI today is size/usage only. Change `deviceCreateBufferDescribed` JNI signature |
| F5 | `gpu-device.create-shader-module` | Guest `label` + `compilation-hints` (empty → none) with existing WGSL `code`. Change `deviceCreateShaderModuleDescribed` JNI signature |
| F6 | `gpu.request-adapter` | Guest `xr-compatible` (`option<bool>`: absent → none). Keep power / fallback / `feature-level`. Keep true async. Change `requestAdapterDescribed` JNI signature |
| F7 | `gpu-adapter.request-device` | Guest `default-queue` (queue `label` is enough this cut; skip nested unused queue fields). Keep first required-feature + required-limits **rep** + device `label`. Keep true async. Full `required-features` list stays named-only. Change `adapterRequestDeviceDescribed` JNI signature |
| F8 | Dawn consume pipeline constants | `GPUComputeState` / `GPUVertexState` / `GPUFragmentState` take `descriptor.*.constants` when the androidx type has a constants slot. Do **not** re-cut the `record-gpu-pipeline-constant-value` resource. Remaining sentinel: the no-constants `GPUComputeState(` block in `DawnWasiWebGpuHost.kt` |
| F9 | Dawn consume required-limits | Map `DeviceDescriptor.requiredLimits` into Dawn `GPUDeviceDescriptor` / `GPULimits` (skip keys androidx does not expose). Do **not** re-cut the `record-option-gpu-size64` resource. Remaining sentinel: `GPUDeviceDescriptor(` with only `label` + callbacks |

Copy: deepen the **existing** described wrap (`[method]gpu-device.create-buffer` + `jvm::exp_*_described` + `attach*` / `ForwardingHostCallbacks`). Do not add a second wrap.

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| SupportedLimits handle-0 | User named handle-0 / `SupportedLimits` / `reserved as null` | Guest `get-supported-limits` must not pass `GpuHandle(0)` into host `Handles`. Prefer a live adapter/device rep, or a dedicated “query adapter, device absent” SPI that does not construct `GpuHandle(0)`. **Not** a limits first-cut re-hang. One short ABI PR. Instruments that currently fail with `handle 0 is reserved as null` should pass |
| Full `required-features` list | User named it | `request-device` forwards **all** required features, not only `.first()`. Do not re-cut the first-feature JNI as a new method |

## File whitelist

- `native/src/cm.rs` — this method’s wrap only (windowed)
- `native/src/jvm.rs` — described JNI for this family (F1–F7: signature **must** change so remaining drops the lane)
- `runtime-api/.../ExperimentalHostCallbacks.kt`
- `host-dawn/.../HostWebGpuBackend.kt` (`ForwardingHostCallbacks` — product path)
- `host-dawn/.../ExperimentalWebGpuBridge.kt` — existing attach for this method
- `host-dawn/.../HostTypes.kt` — add missing record fields for this family only
- `host-dawn/.../DawnWasiWebGpuHost.kt` — F8/F9, or when this family’s record already exists and Dawn still drops it
- `host-dawn/.../AbiCmHostBindings.kt` and Cpu/Dawn hosts **only** if `WasiWebGpuHost` cannot already take the Kotlin record
- `fixtures/w1/webgpu_method_<slug>.{wat,wasm}` — guest values; re-parse/validate `cm-async,component-model`
- `native/tests/wasi_webgpu_method/<slug>.rs` — **assert lifted fields**; duplicate WIT types locally (no `use crate::webgpu_abi`)
- `smoke-app/.../WasiWebGpuMethod*InstrumentedTest.kt` — only if that attach already exists for the method
- `changelog/unreleased/<yyyy-mm-dd>-semantics-<slug>.md` (three bullets)
- `fixtures/w1/README.md` — those table rows

Do not add files under `docs/archive/`. Tests **must not** `use crate::webgpu_abi`. Register `resource()` **before** wraps that use that `Resource<T>`. Use guest `rep` when `!= 0`; do not rebuild adapter → device → encoder when the handle is live.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module>
```

Root `bash ./gradlew :runtime-api:compileKotlin` if Kotlin callbacks changed. Named handle-0: that instrument family only.

PR title: `feat(webgpu): L2 <resource> <family> guest fields to host` (Dawn consume: `feat(webgpu): L2 dawn consume <constants|required-limits>`; handle-0: `fix(webgpu): supported-limits reject handle 0`). Label `enhancement` (handle-0: `bug`).

User prompt that works: “follow `docs/agent/webgpu-guest-semantics.md`” or name the lane (`F1 blend/cull`, `F4 mapped buffer`, `Dawn constants`).
