# Agent playbook: wasi:webgpu midterm lanes

**English** | [中文](webgpu-midterm.zh.md)

Use this **after** default S6+ shape remaining is 0 (canvas omitted) **and** default semantic-L2 remaining host-fixed is 0. Do **not** re-cut label/limits/create-sampler.

Pin: `wasi:webgpu@0.3.0-rc.2` at [`third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit).

Earlier queues: [`webgpu-shape-slice.md`](webgpu-shape-slice.md), [`webgpu-semantic-l2.md`](webgpu-semantic-l2.md). **One lane, one PR.** Never mix A+B+C+D. Never mix a shape hang and an L2 deepen.

## Select the cut

If the user named a lane or `[method]` list, keep **one** family from that list. Otherwise:

```powershell
.\scripts\webgpu-midterm-remaining.ps1
```

No `pwsh`: `python3 ./scripts/webgpu-midterm-remaining.py` (same flags: `--all`, `--include-records`).

Do the printed **Next:** line. Do not Grep `cm.rs` for a queue. Pass `-All` only to list leftover names; pass `-IncludeRecords` only if the user named `record-*` / pipeline constants.

Default order: **A1 → A2 → A3 → A4 → B1 → B2 → B3**. Lane C and D never auto-select.

## Hard bans

- Do **not** WebFetch / clone wasi-webgpu. Grep the vendored WIT.
- Do **not** create, reopen, or request GitHub Issues (or GitHub Discussions used as an issue tracker) on any upstream repository, including [wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) and Wasmtime. No `gh issue create`. Record Android facts only in this repo.
- Do **not** read `native/src/cm.rs`, `native/src/jvm.rs`, `native/src/webgpu_abi.rs`, `ExperimentalHostCallbacks.kt`, `ExperimentalWebGpuBridge.kt`, `HostWebGpuBackend.kt`, `WasiWebGpuHost.kt`, Cpu/Dawn hosts, or RFCs without an offset. Grep the symbol, then Read ~80 lines.
- Do **not** open a third native/Kotlin file to discover a template. Copy the **one** stack in this page for that lane.
- Do **not** edit hub files: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** run full `cargo test --tests` or device instruments unless this PR **is** Lane D.
- Do **not** register experimental `surface-*` as product WIT. Host may call existing `exp_surface_*` from Kotlin; guest names stay `gpu-canvas-context.*`. No wasi-gfx (NG-9). No CTS / compliance claim (NG-5).
- Do **not** add `HostArg` variants. Existing `Int` / `Long` / `Str` / `Ints` are enough. Skip string fields unless this PR is only that need.
- Do **not** rewrite `fixtures/w1/README.md` **Transitional:**. Table row + two `wasm-tools` lines only.
- PowerShell: no bash `&&`, no bash HEREDOC. `git commit` / `gh pr create` use `@"..."@`.

## Lane A — WG-6 `gpu-canvas-context`

WIT has **no** `present` and **no** canvas constructor. Test-only `get-canvas-context` (like `get-gpu`) is required; it is **not** product WIT.

Grep WIT: `resource gpu-canvas-context` (four methods). Records: `gpu-canvas-configuration` (borrow `gpu-device` + `format` + options), `gpu-canvas-configuration-owned`.

| PR | Methods | DoD |
|----|---------|-----|
| A1 shape | all four names, lift-only JNI OK | Hung `[method]` + ABI records at **end** of `webgpu_abi.rs` + `get-canvas-context` |
| A2 L2 | `configure` only | Guest `device.rep` + `format` (+ `usage` if `HostArg::Int`). Drop view-formats / color-space / tone-mapping / alpha-mode to a follow-up |
| A3 L2 | `get-current-texture`; `unconfigure` may ride | Return `own<gpu-texture>` from **guest** context rep; `unconfigure` void on that rep |
| A4 L2 | `get-configuration` | `option<gpu-canvas-configuration-owned>`; do not fold into A2 |

Copy (A1): void → `native/tests/wasi_webgpu_method/render_bundle_encoder_pop_debug_group.rs`; record-in → `create_sampler.rs`; `own<gpu-texture>` → `device_queue.rs` (own resource); `option` → `request_adapter.rs` (drop async). Attach: empty callbacks / existing attach like shape-slice.

Copy (A2–A4): described stack = `[method]gpu-device.create-buffer` + `jvm::exp_create_buffer_described` + `attachCreateBuffer`. Host glue grep: `exp_surface_configure` / `exp_surface_get_view` (do not add product `surface-*` wraps).

## Lane B — S1–S3 guest fields

Already hung + true async. Gap is **discarded guest args** / rebuilt handles. Skip unless remaining prints B* or the user named them.

| PR | Method | DoD |
|----|--------|-----|
| B1 | `gpu-device.queue` | `exp_device_get_queue(device.rep)` when `rep != 0`. **No** `request-adapter` → `request-device` rebuild |
| B2 | `gpu-adapter.request-device` | Keep `func_wrap_concurrent` + `result`. First cut: ignore `required-limits` (Lane C) and string `label`. Optional: one `required-features` enum via `HostArg::Int` / `Ints` |
| B3 | `gpu.request-adapter` | Keep concurrent + `option` / `none` if rep 0. First cut: `power-preference` + `force-fallback-adapter` as `Int`. Skip `feature-level` string |

Copy: existing wraps (Grep `"[method]gpu-device.queue"` / `request-device` / `request-adapter`). JNI today: `exp_device_get_queue` `(I)I`, `exp_adapter_request_device` `(I)I`, `exp_request_adapter` `()I` — add `*_described` siblings; do not replace true async with sync. Tests: `native/tests/wasi_webgpu_method/{device_queue,request_device,request_adapter}.rs` — **assert lifted fields**, still duplicate WIT types locally (no `use crate::webgpu_abi`).

## Lane C — `record-*` maps

**Do not cut** unless the user named `record-*` or a pipeline-create L2 needs guest constants/limits maps. `-IncludeRecords` on the remaining script.

Two resources, **one resource per PR**, then split JNI family:

| Family | Methods |
|--------|---------|
| mutate | `add`, `get`, `has`, `remove` |
| iterate | `keys`, `values`, `entries` |

Copy: existing `native/tests/wasi_webgpu_method/record_gpu_pipeline_constant_value_add.rs` (and sibling `record_option_gpu_size64_*`). Wraps are lift-only today (`_key` ignored). Described JNI: `HostArg::Str` + `Long`/`Ints` as needed. Do not deepen `gpu-device.create-*-pipeline` in the same PR.

## Lane D — citable Dawn slice + WG-5 note

**Manual.** Only if the user said 真机 / 可引用 / WG-5 / upstream.

- **No new WIT names.** Chain already-described methods (buffer/texture + encoder + one render **or** compute pass + `queue.submit`).
- Guest stays canonical `[method]` imports. Dawn path: `DawnWasiWebGpuHost` / default bundle — not Cpu as the cited backend.
- One instrument test is enough. Do **not** claim CTS or a compliant product.
- Android-specific host facts (threads, JNI, Bionic, adapter `none`) stay in this repo: `changelog/unreleased/` plus [`docs/mapping/threading-android.md`](../mapping/threading-android.md) if the pump changed. **Do not** open wasi-webgpu / Wasmtime issues. **Do not** add roadmap “Upstream issue” rows.

## File whitelist

**A1:** `cm.rs` wraps + `get-canvas-context`; `webgpu_abi.rs` records at EOF; fixtures `webgpu_method_canvas_context_<slug>.{wat,wasm}`; `native/tests/wasi_webgpu_method/<slug>.rs` + `main.rs` `mod`; smoke instrument; `ExperimentalWebGpuBridge.kt` empty attach if needed; changelog fragment; `fixtures/w1/README.md` table row.

**A2–A4 / B / C:** same as [`webgpu-semantic-l2.md`](webgpu-semantic-l2.md) whitelist (described JNI + callbacks + Cpu/Dawn **only** if Host API cannot take the scalars).

**D:** one fixture **or** one instrument; changelog. No hub files. No `docs/archive/`. No upstream GitHub issues.

## Narrow tests

A–C (every module in the PR):

```powershell
cd native
cargo check --locked --lib
cargo test --locked --test wasi_webgpu_method -- --test-threads=1 <module_a> <module_b>
```

Root `bash ./gradlew :runtime-api:compileKotlin` if Kotlin callbacks changed. Lane D: that instrument only.

PR titles, label `enhancement` (D docs-only: `documentation`):

- A1: `feat(webgpu): S6+ gpu-canvas-context take WIT types`
- A2–A4 / B / C: `feat(webgpu): L2 <resource> <family> guest fields to host`
- D: `feat(webgpu): cite Dawn <render|compute> slice` or `docs(webgpu): WG-5 <local> note`

User prompt that works: “follow `docs/agent/webgpu-midterm.md`” or name the lane (`A2 configure`, `B1 queue`, `record-* mutate`, `Lane D compute`).
