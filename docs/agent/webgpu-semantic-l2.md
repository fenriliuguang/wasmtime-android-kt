# Agent playbook: wasi:webgpu semantic L2

**English** | [中文](webgpu-semantic-l2.zh.md)

Use this **after** product `[method]` names are hung (S6+ shape). A cut forwards guest fields through JNI into the existing Kotlin/Dawn host (`WasiWebGpuHost`).

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

Shape hangs: [`webgpu-shape-slice.md`](webgpu-shape-slice.md). Do **not** mix a shape hang and an L2 deepen in one PR.

## Batching (caller, then JNI family)

Do **not** open one PR per `[method]`, and do **not** dump every method on a resource into one PR.

1. **Primary key = caller** (`gpu-render-pass-encoder`, `gpu-device`, …). Adjacent `cm.rs` wraps + one attach family.
2. **Split key = JNI family** (same described callback shape: scalars vs buffer-borrow vs `list`/`result`). Typical PR: **2–4** methods.
3. **Local loop:** one method at a time — implement, `wasm-tools` if wat changed, `cargo check --locked --lib`, filtered `wasi_webgpu_method`, **commit**, then the next method in the same family.
4. **PR** when that JNI family is done. Serial: merge (or wait) before the next family on the same hot files.

Same WIT verb on a **different** resource is a **different** PR (`gpu-compute-pass-encoder.draw` ≠ `gpu-render-pass-encoder.draw`). Copy the family later; do not deepen three encoders at once.

Lift-only getters (`texture.width`, `buffer.size`, …) are a later lane (Dawn **reads**). Do not mix them into a host-fixed write family unless the user named them.

### Example: `gpu-render-pass-encoder` (do not ship all nine together)

Today several WIT names share one host-fixed JNI (`renderPassDraw` also covers indexed/indirect; `renderPassSetVertexBuffer` also covers index-buffer). Real L2 must use the guest **`pass.rep`** (and counts / slots / buffers). Do **not** keep rebuilding adapter → encoder → `begin-render-pass-clear` when `rep != 0`.

| PR | Methods | Why this family |
|----|---------|-----------------|
| A — scalar draw | `draw`, `draw-indexed`; add `draw-indirect` / `draw-indexed-indirect` only if they share the same described ints + buffer handle layout | `u32` / `option<u32>` (indirect adds buffer borrow — drop to a follow-up if JNI grows) |
| B — pipeline + buffers | `set-pipeline`, `set-vertex-buffer`, `set-index-buffer` | borrow + slot/format + `option<u64>` |
| C — bind group | `set-bind-group` **alone** | `option<list<…>>` + `result<_, set-bind-group-error>` (may need more than `HostArg` Int/Long) |
| Rider | `end` | void; call `end` on **guest** `pass.rep`. Attach at the tail of A or B, not its own mega-PR |

Do **not** fold `set-viewport` / `set-scissor-rect` / occlusion / debug / `set-immediates` into A–C.

### Default first batch (no names from the user)

**`[method]gpu-device.create-sampler`** (singleton family: optional sampler record, scalar/enum fields, no new `HostArg` string).

Then prefer other **host-fixed** creates on `gpu-device` / `gpu-texture.create-view` before pass-encoder families. Skip S1–S3 (`queue` / `request-adapter` / `request-device`). Skip `create-shader-module` and pipeline **creates** until `HostArg` can pass strings / nested records.

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu. Grep the vendored WIT.
- Do **not** read `native/src/cm.rs`, `native/src/jvm.rs`, `native/src/webgpu_abi.rs`, `ExperimentalHostCallbacks.kt`, `ExperimentalWebGpuBridge.kt`, or `HostWebGpuBackend.kt` without an offset. Grep the method / JNI name, then Read ~80 lines.
- Do **not** open a third native/Kotlin file to discover the template. Copy the **one** gold stack below.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** run full `cargo test --tests` or device instruments. Narrow commands below.
- Do **not** deepen `gpu-canvas-context` (WG-6). Do **not** add `HostArg` string/bytes variants unless the user named a string-bearing method (or family C requires it — then that PR is only bind-group + `HostArg`, nothing else).
- Do **not** rewrite the long **Transitional:** paragraph in `fixtures/w1/README.md`. Update **existing table rows** for fixtures in the batch.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

## Select the cut

If the user listed `[method]` names, keep **one JNI family** from that list (split if they mixed A+B+C). Otherwise:

```powershell
.\scripts\webgpu-semantic-l2-remaining.ps1
```

Prefer the **host-fixed** list. Group by resource, then take **one** family (table above for render-pass; same idea for compute-pass / bundle-encoder as later PRs).

Pass `-IncludeAll` for labels / limits / records. Those are not the default lane.

## What “described L2” means

| Layer | Must change |
|-------|-------------|
| Guest fixture | Pass a **non-host-fixed** scalar (enum/flags/`u32`/`u64`) the old JNI ignored |
| `cm.rs` wrap | Lift WIT args; pass fields into `jvm::exp_*_described`; use table reps (`pass.rep`, buffer/pipeline reps), not a smoke rebuild when `rep != 0` |
| `jvm.rs` | `exp_*_described` using existing `HostArg::Int` / `HostArg::Long` only (one family, not one function per method if they already share JNI) |
| `ExperimentalHostCallbacks` | Described method(s) for that family, default `unsupported(...)` |
| `ForwardingHostCallbacks` | Wire to `AbiCmHostBindings` / `WasiWebGpuHost` |
| `ExperimentalWebGpuBridge` attach | Override **Described** callbacks; do not leave empty `ExperimentalHostCallbacks {}` |

Done today (do not re-cut): `create-buffer`, `create-texture`, `buffer.map-async` (`*_described` JNI).

## Copy sources (one stack)

| Piece | Copy |
|-------|------|
| Native wrap + JNI | `[method]gpu-device.create-buffer` + `jvm::exp_create_buffer_described` |
| Callback + attach | `deviceCreateBufferDescribed` + `ExperimentalWebGpuBridge.attachCreateBuffer` |
| Forwarding | `HostWebGpuBackend.kt` `deviceCreateBufferDescribed` |
| Native test | `native/tests/wasi_webgpu_method/create_buffer.rs` (duplicate WIT types locally; **assert guest fields** in the test wrap) |
| Fixture | Existing `webgpu_method_<slug>.wat` per method — change constants, do not invent a second wasm |

Tests **must not** `use crate::webgpu_abi`. `gpu-texture-format` may use `crate::texture_format::GpuTextureFormat`.

Register `resource()` **before** `func_wrap` that returns/takes that `Resource<T>`.

## File whitelist (typical batch)

- `native/src/cm.rs` — wraps in this family only (windowed)
- `native/src/jvm.rs` — described JNI for this family
- `runtime-api/.../ExperimentalHostCallbacks.kt` — family methods
- `host-dawn/.../HostWebGpuBackend.kt` — `ForwardingHostCallbacks`
- `host-dawn/.../ExperimentalWebGpuBridge.kt` — existing attach
- `host-dawn/.../AbiCmHostBindings.kt` and `WasiWebGpuHost` / Cpu / Dawn **only** if the Host API cannot already take the scalars
- `fixtures/w1/webgpu_method_<slug>.{wat,wasm}` — guest values + re-parse/validate `cm-async,component-model`
- `native/tests/wasi_webgpu_method/<slug>.rs` — assert lifted fields
- `smoke-app/.../WasiWebGpuMethod*InstrumentedTest.kt` — Described override
- `changelog/unreleased/<yyyy-mm-dd>-l2-<slug>.md` (three bullets, name the family)
- `fixtures/w1/README.md` — update those table rows

Do not add files under `docs/archive/`. Do not read RFCs unless JNI `HostArg` or the WIT pin changes.

## Narrow tests

After **each** method locally, and again before the PR, with every module in the family:

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module_a> <module_b>
```

PR: `feat(webgpu): L2 <resource> <family> guest fields to host`, label `enhancement`.

User prompt that works: list the family `[method]` names (or “render-pass draw family”) and “follow `docs/agent/webgpu-semantic-l2.md`”.
