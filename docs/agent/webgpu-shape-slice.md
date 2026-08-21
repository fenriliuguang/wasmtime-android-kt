# Agent playbook: wasi:webgpu shape slice

**English** | [中文](webgpu-shape-slice.zh.md)

Use this for S6+ product `[method]` slices. Do **not** treat each cut as a research project.

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu while the pin is unchanged. Grep the vendored WIT.
- Do **not** create, reopen, or request GitHub Issues (or GitHub Discussions used as an issue tracker) on any upstream repository, including wasi-webgpu and Wasmtime. No `gh issue create`.
- Do **not** read `native/src/cm.rs`, `native/src/webgpu_abi.rs`, or `ExperimentalWebGpuBridge.kt` without an offset. Grep the method name, then Read ~80 lines around the hit.
- Do **not** open a third native/Kotlin test to discover the template. Copy one source below.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed. Never format `command_encoder_finish.rs` / `create_command_encoder.rs` / `queue_submit.rs` unless they are the slice.
- Do **not** run full `cargo test --tests` or device instruments. Narrow commands below.
- Do **not** add experimental JNI or a new host-fixed `u32` product surface. L2 may stay lift-only (lift guest types, ignore fields).
- Do **not** mix `gpu-canvas-context` into this playbook. Canvas is [`webgpu-midterm.md`](webgpu-midterm.md) Lane A (script `-IncludeCanvas` is not a default shape cut).
- Do **not** rewrite the long **Transitional:** paragraph in `fixtures/w1/README.md`. Append a **table row** and two `wasm-tools` lines only.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

## Select the cut

If the user listed `[method]` names, use that list. Otherwise:

```powershell
.\scripts\webgpu-shape-remaining.ps1
```

Prefer a cluster of same-shape methods (all void destroy, all `label`/`set-label`, …). Keep the PR one thing. When picking from the script, skip `record-*` and `gpu-supported-limits.*` unless the user asked for that cluster.

## Copy sources (one each)

| Guest shape | Copy native | Copy Kotlin attach |
|-------------|-------------|--------------------|
| void + borrow (destroy / debug) | `native/tests/wasi_webgpu_method/render_bundle_encoder_pop_debug_group.rs` | `ExperimentalWebGpuBridge.attachRenderBundleState` |
| `own<resource>` + record | `native/tests/wasi_webgpu_method/create_sampler.rs` | lift-only empty callbacks, same as `attachRenderBundleState` |
| `result<own<resource>, E>` | `native/tests/wasi_webgpu_method/create_compute_pipeline_async.rs` (drop the oneshot if the WIT method is sync) | lift-only empty callbacks |
| `result<_, E>` (no own) | `native/tests/wasi_webgpu_method/buffer_unmap.rs` | matching existing attach if L2 is used |

Tests **must not** `use crate::webgpu_abi`. Duplicate WIT types locally. `gpu-texture-format` may use `crate::texture_format::GpuTextureFormat`.

Register `resource()` **before** `func_wrap` that returns/takes that `Resource<T>`.

## File whitelist (typical cut)

- `native/src/cm.rs` — wrap only (windowed edit)
- `native/src/webgpu_abi.rs` — new records/enums at file end if needed
- `fixtures/w1/webgpu_method_<slug>.{wat,wasm}` — parse + validate `cm-async,component-model`
- `native/tests/wasi_webgpu_method/<slug>.rs` + `main.rs` `mod` (ASCII sort)
- `smoke-app/.../WasiWebGpuMethod*InstrumentedTest.kt`
- `host-dawn/.../ExperimentalWebGpuBridge.kt` — empty `ExperimentalHostCallbacks` only if no existing attach fits
- `changelog/unreleased/<yyyy-mm-dd>-<slug>.md` (three bullets, match recent S6+ fragments)
- `fixtures/w1/README.md` — table row + parse/validate commands

Do not add files under `docs/archive/`. Do not read RFCs unless the pin or shape gate changes.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module_a> <module_b>
```

PR: `feat(webgpu): S6+ <cluster> take WIT types`, label `enhancement`.

When `.\scripts\webgpu-shape-remaining.ps1` prints **Remaining: 0**, stop hanging names. Default semantic L2 is [`webgpu-semantic-l2.md`](webgpu-semantic-l2.md). Canvas / S1–S3 descriptors / records / cite: [`webgpu-midterm.md`](webgpu-midterm.md).

User prompt that works: list the `[method]` names and “follow `docs/agent/webgpu-shape-slice.md`”.
