# GPU host: vendor path (Dawn)

**English** | [中文](blocked-gpu-host.zh.md)

**Decision (2026-08-17):** Dawn enters this repo by **vendoring the Kotlin host mapping**. The Dawn **native library** is **not** copied into git; it stays `androidx.webgpu:webgpu` (Google Maven).

Until the vendor PR lands, `:host-dawn` still resolves unpublished mavenLocal coordinates (stop-sign for GPU builds only).

## 1. Chosen form

Copy these sources from `wasi-webgpu-jvm-mvp` into this repo as **implementation of `:host-dawn`** (sibling module `:host-dawn-impl`, or the same module). Keep existing SPI / WIT lowering here.

| Take | From mvp | Role |
|------|----------|------|
| `host-api` Kotlin | `host-api/src/main/kotlin/…/host/` | `WasiWebGpuHost`, `CpuWasiWebGpuHost`, descriptors |
| `DawnWasiWebGpuHost` | `host-webgpu/src/main/java/…/dawn/` | androidx.webgpu adapter |
| `AbiCmHostBindings` | `abi-cm/src/main/kotlin/…/abicm/` | L2 → JNI callback table |

| Depend (published) | Coordinate |
|--------------------|------------|
| Dawn `.so` + AndroidX WebGPU Java | `androidx.webgpu:webgpu:1.0.0-alpha05` (pin in `gradle/libs.versions.toml`; bump with changelog) |

`:host-dawn` then `implementation(project(":host-dawn-impl"))` (or in-module sources) + `api`/`implementation` of `androidx.webgpu`. Drop `libs.wasi.webgpu.host.*` / `abi-cm`.

Public product surface stays `:host-dawn` / `:android-webgpu` / `WebGpuBackend`. `WasiWebGpuHost` becomes an impl detail (package may stay `…experimental…` for the first copy to keep a small diff).

Record Dawn / AndroidX licenses in [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) when the sources land.

## 2. Explicitly out

- New Dawn **product** repository  
- Publishing `io.github.fenriliuguang.wasi.webgpu.experimental:*` as this repo’s contract  
- Git-adding `libwebgpu_c_bundled.so` (that file already ships inside `androidx.webgpu`)  
- Whole mvp tree: wasmtime4j, `runtime-wasmtime`, cube Demo, `abi-mvp`, experimental WIT re-export  
- Implementing `WebGpuBackend` in another repo (cycle)

## 3. Until the copy PR

`:host-dawn` still needs mavenLocal:

| Gradle alias | Coordinates |
|--------------|-------------|
| `libs.wasi.webgpu.host.api` | `io.github.fenriliuguang.wasi.webgpu.experimental:host-api:0.1.0-experimental` |
| `libs.wasi.webgpu.host.webgpu` | `…:host-webgpu:0.1.0-experimental` (pulls androidx.webgpu + wrapper) |
| `libs.wasi.webgpu.abi.cm` | `…:abi-cm:0.1.0-experimental` |

Call sites: `ExperimentalWebGpuBridge`, `GpuBackends`. Core `:runtime-jni` / `:android` do **not** depend on these.

`settings.gradle.kts` still lists `mavenLocal()` for that transitional resolve.

## 4. Layout (already landed)

[`scheme/rfc-pluggable-gpu-backend.md`](scheme/rfc-pluggable-gpu-backend.md): L1 has no Dawn; `:host-dawn` + bundle `:android-webgpu`; unwired `request-adapter` → `none`. SPI in `runtime-api`.
