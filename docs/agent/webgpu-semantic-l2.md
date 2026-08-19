# Agent playbook: wasi:webgpu semantic L2

**English** | [中文](webgpu-semantic-l2.zh.md)

Use this **after** product `[method]` names are hung (S6+ shape). Each cut forwards **one** WIT method’s guest fields through JNI into the existing Kotlin/Dawn host (`WasiWebGpuHost`).

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

Shape hangs: [`webgpu-shape-slice.md`](webgpu-shape-slice.md). Do **not** mix a shape hang and an L2 deepen in one PR.

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu. Grep the vendored WIT.
- Do **not** read `native/src/cm.rs`, `native/src/jvm.rs`, `native/src/webgpu_abi.rs`, `ExperimentalHostCallbacks.kt`, `ExperimentalWebGpuBridge.kt`, or `HostWebGpuBackend.kt` without an offset. Grep the method / JNI name, then Read ~80 lines.
- Do **not** open a third native/Kotlin file to discover the template. Copy the **one** gold stack below.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** run full `cargo test --tests` or device instruments. Narrow commands below.
- Do **not** deepen `gpu-canvas-context` (WG-6). Do **not** add `HostArg` string/bytes variants unless the user named a string-bearing method.
- Do **not** batch methods. **One** product `[method]` per PR (async twin such as `create-*-pipeline-async` may share the same described JNI).
- Do **not** rewrite the long **Transitional:** paragraph in `fixtures/w1/README.md`. Update the **existing table row** for that fixture (and parse/validate lines only if the wat name changed).
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

## Select the cut

If the user listed a `[method]` name, that is the cut. Otherwise:

```powershell
.\scripts\webgpu-semantic-l2-remaining.ps1
```

Prefer the **host-fixed** list (already calls `jvm::exp_*` with ignored guest fields). Skip `gpu.request-adapter` / `gpu-adapter.request-device` / `gpu-device.queue` (S2/S3/S1 already product L2). Skip `create-shader-module` and pipeline creates until HostArg can pass strings / nested records.

Default first pick when the user did not name one: **`[method]gpu-device.create-sampler`**.

Pass `-IncludeAll` for labels / limits / records. Those are not the default lane.

## What “described L2” means

| Layer | Must change |
|-------|-------------|
| Guest fixture | Pass a **non-host-fixed** scalar (enum/flags/`u32`/`u64`) the old JNI ignored |
| `cm.rs` wrap | Lift the WIT record/args; pass fields into `jvm::exp_*_described` (keep the old `exp_*` for other attaches) |
| `jvm.rs` | One new `exp_*_described` using existing `HostArg::Int` / `HostArg::Long` only |
| `ExperimentalHostCallbacks` | One new method, default `unsupported(...)` |
| `ForwardingHostCallbacks` | Wire the new method to `AbiCmHostBindings` / `WasiWebGpuHost` (Host types already exist when possible) |
| `ExperimentalWebGpuBridge` attach | Override the **Described** callback; do not leave empty `ExperimentalHostCallbacks {}` |

Done today (do not re-cut): `create-buffer`, `create-texture`, `buffer.map-async` (`*_described` JNI).

Lift-only getters (`texture.width`, `buffer.size`, …) are a later lane: they need Dawn **reads**, not descriptor writes. Do not start those unless the user named one.

## Copy sources (one stack)

| Piece | Copy |
|-------|------|
| Native wrap + JNI | `[method]gpu-device.create-buffer` + `jvm::exp_create_buffer_described` |
| Callback + attach | `deviceCreateBufferDescribed` + `ExperimentalWebGpuBridge.attachCreateBuffer` |
| Forwarding | `HostWebGpuBackend.kt` `deviceCreateBufferDescribed` |
| Native test | `native/tests/wasi_webgpu_method/create_buffer.rs` (duplicate WIT types locally; **assert guest fields** in the test wrap) |
| Fixture | Existing `webgpu_method_<slug>.wat` for that method — change constants, do not invent a second wasm |

Tests **must not** `use crate::webgpu_abi`. `gpu-texture-format` may use `crate::texture_format::GpuTextureFormat`.

Register `resource()` **before** `func_wrap` that returns/takes that `Resource<T>`.

## File whitelist (typical cut)

- `native/src/cm.rs` — existing wrap only (windowed)
- `native/src/jvm.rs` — one `exp_*_described`
- `runtime-api/.../ExperimentalHostCallbacks.kt` — one method
- `host-dawn/.../HostWebGpuBackend.kt` — `ForwardingHostCallbacks` override
- `host-dawn/.../ExperimentalWebGpuBridge.kt` — existing attach
- `host-dawn/.../AbiCmHostBindings.kt` and `WasiWebGpuHost` / Cpu / Dawn **only** if the Host API cannot already take the scalars
- `fixtures/w1/webgpu_method_<slug>.{wat,wasm}` — guest values + re-parse/validate `cm-async,component-model`
- `native/tests/wasi_webgpu_method/<slug>.rs` — assert lifted fields
- `smoke-app/.../WasiWebGpuMethod*InstrumentedTest.kt` — same attach name, Described override
- `changelog/unreleased/<yyyy-mm-dd>-l2-<slug>.md` (three bullets)
- `fixtures/w1/README.md` — update that table row

Do not add files under `docs/archive/`. Do not read RFCs unless JNI `HostArg` or the WIT pin changes.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module>
```

PR: `feat(webgpu): L2 <method> guest fields to host`, label `enhancement`.

User prompt that works: name one `[method]` and “follow `docs/agent/webgpu-semantic-l2.md`”.
