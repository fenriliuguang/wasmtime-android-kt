# Agent playbook: guest compute / 3D pipeline marshalling

**English** | [中文](webgpu-guest-pipeline.zh.md)

**Closed.** P1–P5 and the named first-cuts on this page are done. Current queue: [`webgpu-guest-semantics.md`](webgpu-guest-semantics.md) (`.\scripts\webgpu-guest-semantics-remaining.ps1`). Do **not** re-hang names, re-cut labels / limits / `create-sampler` first-cut, or re-cut canvas first-cut (`device`+`format`+`usage`).

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

RFCs (do not rediscover policy by reading them mid-cut): [`guest-shape.md`](../scheme/guest-shape.md), [`rfc-ecosystem-contribution.md`](../scheme/rfc-ecosystem-contribution.md), [`rfc-pluggable-gpu-backend.md`](../scheme/rfc-pluggable-gpu-backend.md), [`roadmap-wasi-webgpu.md`](../scheme/roadmap-wasi-webgpu.md). **One lane, one PR.**

## Why this queue

Dawn Kotlin (`DawnWasiWebGpuHost`) already accepts full bind-group / pipeline / render-pass descriptors (see `VectorAddScenario`). Guest WIT → JNI still drops the lists and variants a real compute or 3D mesh needs.

| Guest wants | Blocker today |
|-------------|----------------|
| Compute with storage/uniform | `create-bind-group` `entries = emptyList()`; BGL only **first buffer** entry |
| 3D mesh + MVP | Same bind-group hole; `create-render-pipeline` omits `vertex.buffers` / primitive / depth-stencil; color format JNI `0` → host RGBA8 |
| Depth-tested draw | `begin-render-pass` first color view/load/store only — no depth attachment, no `clearValue` |
| Sampled textures | `create-texture` size/format/usage only; sampler mag/min/`address-mode-u`; view dimension/aspect; BGL has no sampler/texture layout |
| On-screen | No product `present`; `get-canvas-context` is a test ctor (`rep` 0). Experimental `surface-*` is **not** product WIT (NG-9) |

Empty compute `submit` and `@builtin(vertex_index)` offscreen triangles can already work. Do not treat those as this queue’s DoD.

## Select the cut

If the user named a lane or `[method]` list, keep **one** family. Otherwise:

```powershell
.\scripts\webgpu-guest-pipeline-remaining.ps1
```

No `pwsh`: `python3 ./scripts/webgpu-guest-pipeline-remaining.py` (same flags: `--all`).

Do the printed **Next:** line. Do not Grep `cm.rs` for a queue. Default order: **P1 → P2 → P3 → P4 → P5**. Canvas present, leftover sampler/view fields, pipeline constants wiring, S1–S3 leftover descriptor fields, and Dawn **render** cite never auto-select.

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu. Grep the vendored WIT.
- Do **not** create, reopen, or request GitHub Issues (or Discussions-as-issues) on any upstream, including [wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) and Wasmtime. No `gh issue create`. Record Android facts in this repo only.
- Do **not** read `native/src/cm.rs`, `native/src/jvm.rs`, `native/src/webgpu_abi.rs`, `ExperimentalHostCallbacks.kt`, `ExperimentalWebGpuBridge.kt`, `HostWebGpuBackend.kt`, `WasiWebGpuHost.kt`, Cpu/Dawn hosts, or RFCs without an offset. Grep the symbol, then Read ~80 lines.
- Do **not** open a third native/Kotlin file to discover a template. Copy the **one** stack below.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** run full `cargo test --tests` or device instruments unless this PR **is** a named cite / on-screen lane.
- Do **not** register experimental `surface-*` as product WIT. Host may call existing `exp_surface_*` from Kotlin; guest names stay `gpu-canvas-context.*`. No wasi-gfx (NG-9). No CTS / compliance claim (NG-5).
- Do **not** add `HostArg` variants except **P1** may add `HostArg::Longs` if `u64` offset/size vectors cannot pack into existing `Int` / `Long` / `Str` / `Ints` / `Bytes`. That addition is this PR only.
- Do **not** rewrite `fixtures/w1/README.md` **Transitional:**. Table row + two `wasm-tools` lines only.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

`request-adapter` with no backend still returns guest **`none`**. Keep true `async` wraps (`func_wrap_concurrent` + yield) for WIT `async func`.

## Lanes (auto)

JNI sentinels live in `native/src/jvm.rs` (remaining script). Dawn host SPI already takes the Kotlin records — **forward guest fields into existing** `deviceCreateBindGroup` / `deviceCreateBindGroupLayout` / `deviceCreateRenderPipeline` / `commandEncoderBeginRenderPass` / `deviceCreateTexture`. Do not invent a second GPU stack (NG-7).

| PR | Method | DoD |
|----|--------|-----|
| P1 | `gpu-device.create-bind-group` | Guest `entries` (`binding` + `gpu-binding-resource` variant) reach host. Empty list stays valid. Fixture asserts **at least one buffer** entry (handle + binding index), not `emptyList()` |
| P2 | `gpu-device.create-bind-group-layout` | **All** `entries`, not only `.first()`. Buffer types **and** sampler / texture / storage-texture options (absent → none). Fixture: ≥2 buffer entries **or** one buffer + one sampler/texture |
| P3 | `gpu-device.create-render-pipeline` | Guest `vertex.buffers` (stride / step / attributes) + fragment **target format** from the descriptor (stop hardcoding JNI `format = 0`). Drop blend / multisample / pipeline `constants` / full primitive (cull, strip-index) to a named follow-up. Topology may stay TriangleList if unset |
| P4 | `gpu-command-encoder.begin-render-pass` | First color attachment **plus** `depth-stencil-attachment` (view + depth load/store/clear) **plus** color `clear-value` when present. Extra color attachments stay a named follow-up |
| P5 | `gpu-device.create-texture` | Guest `mip-level-count` / `sample-count` / `dimension` (plus existing size/format/usage). `view-formats` / label stay optional drop |

P1 packing (prefer existing `HostArg`): parallel `Ints` for `binding[]`, `kind[]` (0=buffer, 1=sampler, 2=texture-view), `handle[]`; `u64` offset/size as `Longs` **or** hi/lo `Ints`. Indexed add-then-create JNI (like `record-*`) is OK if arrays are too wide — still one WIT wrap.

P2 packing: parallel `Ints` for binding / visibility / kind / buffer-type (and sampler/texture enums). Same “no new `HostArg` unless unavoidable” rule as P1.

Copy: `[method]gpu-device.create-buffer` + `jvm::exp_create_buffer_described` + `attachCreateBuffer` / `ForwardingHostCallbacks`. Deepen the **existing** create-bind-group / layout / render-pipeline / begin-render-pass / create-texture wrap — do not add a second wrap.

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| Sampler / view leftovers | User named them | Sampler: `address-mode-v/w`, mipmap, lod, compare. View: format / mip / array layers |
| Pipeline constants | User named `constants` / `record-*` into create-*-pipeline | Pass `record-gpu-pipeline-constant-value` **rep** into create-compute/render-pipeline; do not re-cut the map resource |
| S1–S3 leftovers | User named `required-limits` / `label` / `feature-level` | B2 `required-limits` via existing `record-option-gpu-size64`; keep true async |
| Canvas present (WG-6) | User said 上屏 / canvas / present | Host-owned Android window behind `gpu-canvas-context` (`configure` / `get-current-texture`). **No** product `surface-*` names. Test-only `get-canvas-context` may stay for fixtures |
| Cite render (WG-5 / L4) | User said 真机 / 可引用 / Lane D render | Chain already-described methods on `DawnWasiWebGpuHost` (buffer/texture + encoder + **render** pass + `queue.submit`). One instrument. No CTS. Notes in `changelog/unreleased/` + [`threading-android.md`](../mapping/threading-android.md) if the pump changed. Compute cite already exists — do not redo it |

## File whitelist

- `native/src/cm.rs` — this method’s wrap only (windowed)
- `native/src/jvm.rs` — described JNI for this family (signature **must** change so remaining drops the lane)
- `runtime-api/.../ExperimentalHostCallbacks.kt`
- `host-dawn/.../HostWebGpuBackend.kt` (`ForwardingHostCallbacks` — product path)
- `host-dawn/.../ExperimentalWebGpuBridge.kt` — existing attach for this method
- `host-dawn/.../AbiCmHostBindings.kt` and Cpu/Dawn hosts **only** if `WasiWebGpuHost` cannot already take the Kotlin record
- `fixtures/w1/webgpu_method_<slug>.{wat,wasm}` — guest values; re-parse/validate `cm-async,component-model`
- `native/tests/wasi_webgpu_method/<slug>.rs` — **assert lifted fields**; duplicate WIT types locally (no `use crate::webgpu_abi`)
- `smoke-app/.../WasiWebGpuMethod*InstrumentedTest.kt` — only if that attach already exists for the method
- `changelog/unreleased/<yyyy-mm-dd>-pipeline-<slug>.md` (three bullets)
- `fixtures/w1/README.md` — those table rows

Do not add files under `docs/archive/`. Tests **must not** `use crate::webgpu_abi`. Register `resource()` **before** wraps that use that `Resource<T>`. Use guest `rep` when `!= 0`; do not rebuild adapter → device → encoder when the handle is live.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module>
```

Root `bash ./gradlew :runtime-api:compileKotlin` if Kotlin callbacks changed. Named cite / on-screen: that instrument only.

PR title: `feat(webgpu): L2 <resource> <family> guest fields to host` (cite: `feat(webgpu): cite Dawn render slice`). Label `enhancement` (docs-only cite note: `documentation`).

User prompt that works: “follow `docs/agent/webgpu-guest-pipeline.md`” or name the lane (`P1 bind-group entries`, `P4 depth attachment`, `canvas present`).
