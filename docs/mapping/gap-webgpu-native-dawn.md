# Gap: `wasi:webgpu` WIT ↔ NativeGpu ↔ Dawn C

**English** | [中文](gap-webgpu-native-dawn.zh.md)

Living map for the **in-process Dawn C** path (`NativeGpu`). Pin: `wasi:webgpu@0.3.0-rc.2`. JNI leftover: [`gap-webgpu-wit-androidx.md`](gap-webgpu-wit-androidx.md). Claim: [`../scheme/claim-010.md`](../scheme/claim-010.md).

`libwebgpu_dawn.so` is packed into Maven `host-dawn` from **0.1.2** (press pin: `--prebuilt`). Cloud CI assemble without the recipe → table-backed. Table-backed boot is still NativeGpu (no ART/JNI).

**Degree**

| Tag | Meaning |
|-----|---------|
| **Dawn** | Guest value reaches Dawn C (`webgpu.h`) when the `.so` is loaded |
| **Table** | Guest value reaches the handle table; Dawn C slot stays 0 (or the C call is missing) |
| **Record** | Packed on the host record; Dawn C has no slot |
| **JNI** | `JniBackend` leftover (`GpuBackends.dawnJni()`) |

## 1. Coverage

| Family | When `.so` loaded | Otherwise |
|--------|-------------------|-----------|
| `request-adapter` / `request-device` / `queue` | **Dawn** (Vulkan adapter; power / fallback / feature-level / required-features / labels on the C call. `required-limits` and `xr-compatible` stay **Record**) | **Table** |
| create-buffer / shader-module / bind-group / layouts / render-pipeline | **Dawn** (blend / depth-stencil / MSAA / pipeline constants on the C ctor) | **Table** |
| command encoder / begin-render-pass (color + optional depth) / draw / set-pipeline / set-bind-group / set-vertex-buffer / finish / submit / write-buffer | **Dawn** | **Table** |
| create-texture / sampler / compute pipeline / compute pass / copies / clear / query-set / render-bundle **recording** / map-async + mapped-range get/set / write-texture / work-done / indexed-indirect / viewport / scissor / blend / stencil / occlusion / adapter `has` + `GetInfo` / **GetLimits** / buffer size·usage·map-state / texture getters / destroy | **Dawn** (compute pipeline constants on the C ctor; copy origin / mip / aspect / layout; `set-bind-group` dynamic offsets). Mapped-range uses `wgpuBufferGetConstMappedRange` / `GetMappedRange`; bundle commands call `wgpuRenderBundleEncoder*`. | **Table** (CPU shadow of writes; offset/size honored; limits getters `1`) |
| Android `ANativeWindow` surface / configure / get-current-texture / present | **Dawn** (Fifo; color-space / tone-mapping **Record**) | **Table** (keep-3 / H8 still) |

`wasi-gfx` `on-pointer-*` / `on-key-*` are host-wired (`Store.postGfxPointer` / `postGfxKey` → bounded gate). Not Dawn C.

## 2. Record holes (not BIND)

| WIT | NativeGpu | Dawn C |
|-----|-----------|--------|
| `gpu-shader-module-descriptor.compilation-hints` | **Record** | no hints slot |
| `gpu-canvas-configuration.color-space` | **Record** | no color-space on `WGPUSurfaceConfiguration` |
| `gpu-canvas-configuration.tone-mapping` | **Record** | no tone-mapping slot |
| `required-limits` / `xr-compatible` | **Record** | no C slot on the request-device / request-adapter call used here |

## 3. Remaining Table (BIND leftover, [#317](https://github.com/fenriliuguang/wasmtime-android-kt/issues/317))

These pin names are registered. `.so` does **not** make them Dawn yet. Auto queue: [`../scheme/nativegpu-remaining.md`](../scheme/nativegpu-remaining.md) (`python3 ./scripts/nativegpu-remaining.py`). Do **not** skip this table because WASI leftover is empty.

| Family | NativeGpu today | Missing / skipped |
|--------|-----------------|-------------------|
| `gpu-supported-limits.*` | **Dawn** `GetLimits` when `.so` loads; Cloud / missing `.so` still `1` | `required-limits` on request-device stays **Record** (NG-7) |
| `compilation-info` / messages | empty list | `wgpuShaderModuleGetCompilationInfo` (`gap: n-compinfo pending`) |
| `pop-error-scope` result | always `ok(none)` | callback type/message not returned to guest (`gap: n-poperr pending`) |
| `on-uncaptured-error` | empty stream | Dawn uncaptured callback not attached (`gap: n-uncaptured pending`) |
| `gpu-error` / `device-lost-info` | empty / unknown | no lost/uncaptured wiring (`gap: n-uncaptured pending`) |
| `wgsl-language-features.has` | always `false` | no Dawn query (`gap: n-wgslfeat pending`) |
| labels except buffer/texture | get `""` / set dropped | `wgpu*SetLabel` not loaded; host table only for buffer/texture (`gap: n-labels pending`) |
| `set-immediates` / debug group·marker | no-op | no C wrappers (`gap: n-immediate pending`) |
| copy origin / mip / aspect / layout | **Dawn** (guest fields reach `texel_tex` / `texel_buf`) | — |
| `set-bind-group` dynamic offsets | **Dawn** (C `usize, *const u32`) | — |
| `queue.submit` canvas recycle | `mark_canvas_gpu_done` without fence | hitch leftover vs D24 `onSubmittedWorkDone` (`gap: n-fence pending`) |

Unwired store: `gpu.request-adapter` → guest **`none`**. `GpuBackends.dawn()` selected → table-backed adapter (not `none`) even without the `.so`.
