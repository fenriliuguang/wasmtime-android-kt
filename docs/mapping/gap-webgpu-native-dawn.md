# Gap: `wasi:webgpu` WIT ↔ NativeGpu ↔ Dawn C

**English** | [中文](gap-webgpu-native-dawn.zh.md)

Living map for the **in-process Dawn C** consume path (`NativeGpu`). Pin: `wasi:webgpu@0.3.0-rc.2`. JNI leftover: [`gap-webgpu-wit-androidx.md`](gap-webgpu-wit-androidx.md). Playbook: [`../agent/native-dawn.md`](../agent/native-dawn.md). Do not treat this page as a cut queue — needles stay in [`../scheme/native-dawn.md`](../scheme/native-dawn.md).

Dawn C `u64` slots stay **0** until a later lane dlopens `libwebgpu_dawn.so`. Table-backed boot is still NativeGpu (no ART/JNI).

**Degree**

| Tag | Meaning |
|-----|---------|
| **Table** | Guest value reaches `NativeGpuHost` handle table; Dawn C slot is 0 |
| **Dawn** | Guest value reaches Dawn C (`webgpu.h`) |
| **Record** | Packed on the host record; Dawn C has no slot |
| **Pending** | Later native-dawn lane |
| **JNI** | Still `JniBackend` leftover (product default until ND-DEFAULT) |

## 1. Coverage (updated ND-REST)

| Family | Degree |
|--------|--------|
| `gpu.request-adapter` / `gpu-adapter.request-device` / `gpu-device.queue` | **Table** |
| Adapter info / features / limits needed to boot | **Table** |
| create-buffer / texture / sampler / shader-module / texture-view | **Table** (shader `compilation-hints` **Record**) |
| bind-group / layouts / pipelines | **Table** (pipeline constants copied onto the pipeline handle; Dawn C slot 0) |
| command encoder / passes / draws / copies / query-sets | **Table** |
| queue submit / write-buffer / write-texture / work-done | **Table** (guest `list<u8>` is one host copy; Dawn C slot 0) |
| Remaining pin `[method]`s (`wasi_webgpu_method` suite) | **Table** (labels / limits getters / WGSL features / map-async / error scopes / lost / compilation-info / render bundles / canvas get-configuration stub; Dawn C slot 0) |
| canvas `ANativeWindow` surface / present | **Pending** ND-SURF |

## 2. Leftover vs Dawn C

| WIT | NativeGpu | Dawn C |
|-----|-----------|--------|
| `gpu-shader-module-descriptor.compilation-hints` | **Record** on `NativeGpuHost` (not copied into Dawn C) | no `WGPUShaderModuleDescriptor` hints slot |
| `gpu-canvas-configuration.color-space` | **Pending** ND-SURF / **Record** if still no slot | no color-space on `WGPUSurfaceConfiguration` at androidx `1.0.0-alpha05` pin |
| `gpu-canvas-configuration.tone-mapping` | **Pending** ND-SURF / **Record** if still no slot | no tone-mapping slot |
| `gpu-canvas-context` configure / get-current-texture (table stub; no `ANativeWindow`) | **Table** until ND-SURF | real surface is ND-SURF |
| Real `ANativeWindow` surface / present | **Pending** ND-SURF | C API `WGPUSurface` from window handle |

Unwired JNI store is unchanged: `gpu.request-adapter` → guest **`none`**. NativeGpu selected (slot set) → table-backed adapter (not `none`).
