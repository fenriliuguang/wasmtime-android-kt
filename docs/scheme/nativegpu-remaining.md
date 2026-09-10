# NativeGpu Remaining Table queue (tracking)

**English** | [中文](nativegpu-remaining.zh.md)

Living **auto** queue for the BIND leftover in [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md) §3. Pin methods are registered; `.so` loaded still does **not** make these Dawn until the matching needle lands.

This is **not** the WASI leftover. `python3 ./scripts/wasi-p3-leftover-remaining.py` is empty and **must not** print `Named-only: … native-dawn re-cuts | Closed` as a reason to skip this table. Auto: run **`python3 ./scripts/nativegpu-remaining.py`**.

NG-5 / NG-7 stay: not CTS; not a second GPU. Record holes (`compilation-hints`, canvas `color-space` / `tone-mapping`, `required-limits` / `xr-compatible`) and gfx named-only (`unconfigure`, timestamped `frame-event`, Lost/Outdated, multi-window) are **never** `Next:`.

Remaining: `python3 ./scripts/nativegpu-remaining.py` (next **PR**, not a long WASI-style fork). A lane drops when its **`gap: n-… pending`** needle leaves **this file**. Do **not** remove a needle without landing that lane’s DoD.

## How to cut

1. From latest `main`. Run `python3 ./scripts/nativegpu-remaining.py`. Do the printed **Next:** only.
2. One PR, one knife. Changelog fragment. Hub freeze: no root `README.md` / `CHANGELOG.md` / `ci.yml` unless the PR *is* this playbook.
3. Cloud has **no** `.so`. Table-backed getters stay `1` / empty when Dawn is missing. `cargo test` must stay green without `libwebgpu_dawn.so`.
4. New tests only as `native/tests/*.rs` (or `native_gpu.rs` `#[cfg(test)]`). Do not edit `ci.yml`. Do not recut `dawn-jni`.
5. Never file GitHub issues on Dawn / Wasmtime / WASI.

`rustc` **1.97.1**. Do not crate-`cargo fmt`.

## Needles (auto order)

<!-- remaining.py greps these exact strings. Keep one per unfinished lane. -->

| Lane | Needle (delete when landed) |
|------|------------------------------|
| N-RFC | landed 2026-09-10 (this playbook / remaining script / leftover pointer) |
| N-LIMITS | landed 2026-09-10 (`wgpuAdapterGetLimits` / `wgpuDeviceGetLimits`; Cloud stays table `1`) |
| N-COPY | landed 2026-09-10 (copy / `write-texture` origin / mip / aspect / buffer layout) |
| N-DYNOFF | landed 2026-09-10 (`set-bind-group` dynamic offsets reach the C `usize, *const u32`) |
| N-COMPINFO | landed 2026-09-10 (`wgpuShaderModuleGetCompilationInfo`; Cloud empty list) |
| N-POPERR | landed 2026-09-10 (`pop-error-scope` returns callback type/message; Cloud `ok(none)`) |
| N-UNCAPTURED | landed 2026-09-10 (uncaptured + lost callbacks; Cloud empty / unknown) |
| N-WGSLFEAT | landed 2026-09-10 (`wgpuInstanceHasWGSLLanguageFeature`; Cloud `false`) |
| N-LABELS | landed 2026-09-10 (`wgpu*SetLabel` + host get for resources other than buffer/texture) |
| N-IMMEDIATE | landed 2026-09-10 (`set-immediates` / debug group·marker C wrappers) |
| N-FENCE | landed 2026-09-10 (`queue.submit` waits `OnSubmittedWorkDone` before canvas recycle) |

## Lanes (auto)

| Commit | Needle | DoD |
|--------|--------|-----|
| **N-RFC** | *(this commit)* | Playbook + remaining script. Leftover Named-only points here. |
| **N-LIMITS** | *(landed)* | Bind `wgpuAdapterGetLimits` / `wgpuDeviceGetLimits`. `gpu-supported-limits.*` read Dawn fields when the `.so` is loaded; missing `.so` still returns `1`. Do **not** fill `request-device` `required-limits` (Record / NG-7). |
| **N-COPY** | *(landed)* | Guest `GpuTexelCopyTextureInfo` / `GpuTexelCopyBufferInfo` origin / mip / aspect / offset / bytes-per-row / rows-per-image reach `texel_tex` / `texel_buf`. Includes `write-texture-with-copy`. |
| **N-DYNOFF** | *(landed)* | `set-bind-group` offsets + start/length slice reach `wgpu*SetBindGroup`. Render pass, compute pass, bundle encoder. |
| **N-COMPINFO** | *(landed)* | `compilation-info` / messages from `wgpuShaderModuleGetCompilationInfo`. |
| **N-POPERR** | *(landed)* | `pop-error-scope` returns the callback type/message, not always `ok(none)`. |
| **N-UNCAPTURED** | *(landed)* | `on-uncaptured-error` + `gpu-error` / `device-lost-info` wired from Dawn callbacks. |
| **N-WGSLFEAT** | *(landed)* | `wgsl-language-features.has` queries Dawn (not always `false`). |
| **N-LABELS** | *(landed)* | `wgpu*SetLabel` / get for resources other than buffer/texture. |
| **N-IMMEDIATE** | *(landed)* | `set-immediates` / debug group·marker C wrappers (not no-op). |
| **N-FENCE** | *(landed)* | `queue.submit` canvas recycle waits on a fence vs D24 `onSubmittedWorkDone`. `remaining.py` empty → table §3 has no pending BIND leftover. |

This amendment: auto previously treated native-dawn as leftover Named-only **Closed**, so this table was never `Next:`. All BIND leftover knives in §3 land on this PR (user: continue remaining defects here). Record holes / gfx named-only stay never `Next:`.

## Named-only (never `Next:`)

| Item | Why |
|------|-----|
| Record holes | gap §2; NG-7 (no second GPU) |
| `required-limits` on `request-device` | Record; GetLimits is N-LIMITS only |
| gfx `unconfigure` / timestamped `frame-event` / Lost/Outdated / multi-window | gfx named-only |
| `dawn-jni` | leftover map; do not recut |
| WASI `L-*` / wasi-testsuite / `wasmtime-wasi` | leftover empty; NG-4 |
| CTS / this-repo 1.0 | NG-5 |

## File whitelist (typical knife)

- `native/src/dawn_c.rs` / `native/src/native_gpu.rs` / `native/src/cm.rs` — this family’s wrappers only
- `native/tests/*.rs` or `native_gpu.rs` tests
- `docs/scheme/nativegpu-remaining.md` — **remove this lane’s needle**
- `docs/mapping/gap-webgpu-native-dawn.md` — one row
- `changelog/unreleased/<yyyy-mm-dd>-n-<slug>.md`

## Narrow tests

```text
cd native && rustup default 1.97.1
cargo test --locked --test wasi_webgpu_method --test native_gpu
python3 ./scripts/nativegpu-remaining.py
```
