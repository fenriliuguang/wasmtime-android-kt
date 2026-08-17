# GPU host: vendor path (Dawn)

**English** | [中文](blocked-gpu-host.zh.md)

**Landed (2026-08-17):** Dawn enters this repo by **vendoring the Kotlin host mapping** into `:host-dawn`. The Dawn **native library** is **not** in git; it is `androidx.webgpu:webgpu:1.0.0-alpha05` (Google Maven). Pin: `gradle/libs.versions.toml` → `webgpu`. Bump with a changelog fragment.

Public product surface stays `:host-dawn` / `:android-webgpu` / `WebGpuBackend`. `WasiWebGpuHost` is an impl detail (package `…experimental…` kept on the first copy).

## 1. In-tree layout

| Path | Role |
|------|------|
| `host-dawn/src/main/kotlin/…/experimental/host/` | `WasiWebGpuHost`, `CpuWasiWebGpuHost`, descriptors |
| `host-dawn/src/main/kotlin/…/experimental/dawn/DawnWasiWebGpuHost.kt` | androidx.webgpu adapter |
| `host-dawn/src/main/kotlin/…/experimental/abicm/` | L2 → JNI callback table |
| `host-dawn/third_party/wasi-webgpu-jvm-mvp/` | MIT license + origin commit of the copy |

`:host-dawn` `api(libs.androidx.webgpu)`. No `mavenLocal()`. No `io.github.fenriliuguang.wasi.webgpu.experimental:*` Maven coordinates.

Source of the copy: sibling tree `wasi-webgpu-jvm-mvp` at commit recorded in [`../host-dawn/third_party/wasi-webgpu-jvm-mvp/ORIGIN.txt`](../host-dawn/third_party/wasi-webgpu-jvm-mvp/ORIGIN.txt).

## 2. Explicitly out

- New Dawn **product** repository  
- Publishing `io.github.fenriliuguang.wasi.webgpu.experimental:*` as this repo’s contract  
- Git-adding `libwebgpu_c_bundled.so` (that file already ships inside `androidx.webgpu`)  
- Whole mvp tree: wasmtime4j, `runtime-wasmtime`, cube Demo, `abi-mvp`, experimental WIT re-export  
- Implementing `WebGpuBackend` in another repo (cycle)

## 3. Licenses

Recorded in [`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md): vendored Host Kotlin (MIT); `androidx.webgpu` (Apache-2.0) including bundled Dawn native.

## 4. Module graph

[`scheme/rfc-pluggable-gpu-backend.md`](scheme/rfc-pluggable-gpu-backend.md): L1 has no Dawn; `:host-dawn` + bundle `:android-webgpu`; unwired `request-adapter` → `none`. SPI in `runtime-api`.
