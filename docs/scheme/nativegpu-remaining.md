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
| N-COMPINFO | gap: n-compinfo pending |
| N-POPERR | gap: n-poperr pending |
| N-UNCAPTURED | gap: n-uncaptured pending |
| N-WGSLFEAT | gap: n-wgslfeat pending |
| N-LABELS | gap: n-labels pending |
| N-IMMEDIATE | gap: n-immediate pending |
| N-FENCE | gap: n-fence pending |

## Lanes (auto)

| Commit | Needle | DoD |
|--------|--------|-----|
| **N-RFC** | *(this commit)* | Playbook + remaining script. Leftover Named-only points here. `nativegpu-remaining.py` after the catch-up knives prints **`Next: N-COMPINFO`**. |
| **N-LIMITS** | *(landed)* | Bind `wgpuAdapterGetLimits` / `wgpuDeviceGetLimits`. `gpu-supported-limits.*` read Dawn fields when the `.so` is loaded; missing `.so` still returns `1`. Do **not** fill `request-device` `required-limits` (Record / NG-7). Remove the needle. |
| **N-COPY** | *(landed)* | Guest `GpuTexelCopyTextureInfo` / `GpuTexelCopyBufferInfo` origin / mip / aspect / offset / bytes-per-row / rows-per-image reach `texel_tex` / `texel_buf` (already in `dawn_c`). Includes `write-texture-with-copy`. Remove the needle. |
| **N-DYNOFF** | *(landed)* | `set-bind-group` offsets + start/length slice reach `wgpu*SetBindGroup` (C already takes `usize, *const u32`). Render pass, compute pass, bundle encoder. Remove the needle. |
| **N-COMPINFO** | `gap: n-compinfo pending` | `compilation-info` / messages from `wgpuShaderModuleGetCompilationInfo`. Remove the needle. |
| **N-POPERR** | `gap: n-poperr pending` | `pop-error-scope` returns the callback type/message, not always `ok(none)`. Remove the needle. |
| **N-UNCAPTURED** | `gap: n-uncaptured pending` | `on-uncaptured-error` + `gpu-error` / `device-lost-info` wired from Dawn callbacks. Remove the needle. |
| **N-WGSLFEAT** | `gap: n-wgslfeat pending` | `wgsl-language-features.has` queries Dawn (not always `false`). Remove the needle. |
| **N-LABELS** | `gap: n-labels pending` | `wgpu*SetLabel` / get for resources other than buffer/texture. Remove the needle. |
| **N-IMMEDIATE** | `gap: n-immediate pending` | `set-immediates` / debug group·marker C wrappers (not no-op). Remove the needle. |
| **N-FENCE** | `gap: n-fence pending` | `queue.submit` canvas recycle waits on a fence vs D24 `onSubmittedWorkDone`. Remove the needle. `remaining.py` empty → table §3 has no pending BIND leftover. |

This amendment (catch-up): auto previously treated native-dawn as leftover Named-only **Closed**, so this table was never `Next:`. N-LIMITS / N-COPY / N-DYNOFF land with the playbook. Later knives stay **one Next per PR**.

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
