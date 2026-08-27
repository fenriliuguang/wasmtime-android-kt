# Gap: `wasi:webgpu` WIT ↔ host ↔ androidx.webgpu

**English** | [中文](gap-webgpu-wit-androidx.zh.md)

Living map for the **closed** P0 host. Pin: `wasi:webgpu@0.3.0-rc.2` ([`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit)). Dawn AAR: `androidx.webgpu:webgpu:1.0.0-alpha05`. Do not treat this page as a cut queue. P0 close-out: [`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md). Current work: [`../agent/product-010.md`](../agent/product-010.md).

**Degree**

| Tag | Meaning |
|-----|---------|
| **Dawn** | Guest value reaches `androidx.webgpu` / Dawn |
| **Record** | JNI packed the WIT field onto a Kotlin `WasiWebGpuHost` record; Dawn ctor has no slot |
| **Shape** | Guest import matches WIT; L2 described JNI |
| **Fixture** | Test-only constructor (`get-gpu`, `get-device`, …) — not product WIT |
| **Flat** | Frozen experimental / dual-register `u32` name — do not extend |
| **Out** | Explicit non-goal (CTS, wasi-gfx, rewrite Dawn) |

## 1. Coverage

| Axis | Degree |
|------|--------|
| `[method]` names vs pin | **Shape** — ~225 resource methods wrapped in `native/src/cm.rs` |
| S1–S5 (`queue`, `request-adapter` `option`, `request-device` `result`, `create-buffer`, `submit` list) | **Shape** + **Dawn** on the product path |
| Leftover optional descriptors (F1–F9) | **Shape** into Kotlin records; Dawn copy where the AAR allows |
| G1–G9 consume + WG-6 guest-drawn compute / 3D / canvas present | **Dawn** (see holes below) |
| Unwired store | `gpu.request-adapter` → guest **`none`** (not a trap) |

## 2. androidx holes (Record, not Dawn)

These are the only P0 fields still dropped on the GPU. Kotlin records keep the guest values. Revisit only when bumping `androidx.webgpu` (changelog the AAR pin). Do **not** re-cut G3 / G8.

| WIT | Kotlin | androidx `1.0.0-alpha05` |
|-----|--------|-------------------------|
| `gpu-shader-module-descriptor.compilation-hints` | `ShaderModuleDescriptor.compilationHints` | `GPUShaderModuleDescriptor` is label + WGSL/SPIR-V only |
| `gpu-canvas-configuration.color-space` | leftover on canvas configure (`colorSpace`, `-1` = absent) | `GPUSurfaceConfiguration` has no color-space slot |
| `gpu-canvas-configuration.tone-mapping` | leftover (`toneMapping`, `-1` = absent) | no tone-mapping slot |

Related slots that **do** exist on this AAR (already consumed): primitive cull/front/strip, blend, multisample, texture `viewFormats`, `requestAdapterWebXROptions`, `GPUDeviceDescriptor.defaultQueue`, color `writeMask`, depth-stencil faces/masks/bias, surface `viewFormats` / `alphaMode`, compute `layout` omitted = auto.

## 3. Host vs WIT (by family)

Guest boundary is the pin. Dawn handles stay `u32` **reps** behind `own`/`borrow`.

| Family | WIT | Host / Dawn | Degree |
|--------|-----|-------------|--------|
| Adapter / device / queue | async `option`/`result`; `required-features` list; `required-limits` map including stage storage keys | Dawn request + `GPULimits` / compatibility-mode setters | **Dawn** |
| Buffer / texture / sampler / views | descriptors + map-async true async | described JNI → Dawn | **Dawn** |
| Shader module | WGSL + label + hints | hints stay on record | **Dawn** code, **Record** hints |
| Pipelines | vertex buffers, blend, MSAA, write-mask, stencil leftovers, constants, `layout: auto` | Dawn ctors; auto = omit layout | **Dawn** |
| Passes / copies / debug / bundles / query-sets | recording commands | described JNI → Dawn/Cpu | **Shape** + **Dawn** |
| Canvas | `configure` device+format+usage+leftovers; `get-current-texture`; guest-drawn present | `GPUSurfaceConfiguration`; present is host `exp_surface_*` internally, guest names stay `gpu-canvas-context.*` | **Dawn** except color-space / tone-mapping **Record** |
| Labels / info getters / supported-limits | WIT getters | described JNI | **Shape** + **Dawn** |
| `get-*` test constructors | not in product WIT | fixtures / instruments | **Fixture** |
| `experimental:webgpu-cm` flats (`device-get-queue`, `queue-submit1`, …) | not pin product names | still registered | **Flat** |

Cpu host (`CpuWasiWebGpuHost`) is a stand-in: VectorAdd shader-text match only. Product GPU path is Dawn.

## 4. Out of scope

| Item | Tag |
|------|-----|
| WebGPU CTS / “compliant wasi:webgpu” | **Out** (NG-5) |
| Product `surface-*` / wasi-gfx as a **P0** re-queue | **Out** (NG-9). Minimal present loop is **`0.1.0`**: [`../scheme/rfc-wasi-gfx-frame-loop.md`](../scheme/rfc-wasi-gfx-frame-loop.md) |
| Second Dawn renderer | **Out** (NG-7) |
| `wasmtime-wasi` as the WebGPU host | **Out** — GPU is `:host-dawn` |

When androidx grows a hole’s ctor argument: copy the existing Kotlin field into Dawn in that pin-bump PR, and update this table. Do not reopen G1–G9 or F1–F9 as queues.
