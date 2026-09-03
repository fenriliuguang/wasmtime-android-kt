# Third-party notices

**English** | [中文](THIRD_PARTY_NOTICES.zh.md)

This repository’s own code: [`LICENSE`](LICENSE) (Apache-2.0) and [`NOTICE`](NOTICE).  
The table below is a **scan summary** of direct and major transitive licenses (`cargo license`, 2026-08-11). Authoritative SPDX is each crate / Maven artifact.

## 1. Rust (`native/`)

| Dependency | Role | SPDX (scan) |
|------------|------|-------------|
| `wasmtime` (+ Cranelift, …) | Engine | Apache-2.0 WITH LLVM-exception |
| `jni` | JNI | Apache-2.0 OR MIT |
| `futures` | async helpers | Apache-2.0 OR MIT |
| `pollster` | blocking driver | Apache-2.0 OR MIT |
| `getrandom` | WASI random CSPRNG | Apache-2.0 OR MIT |

Transitives are mostly **Apache-2.0 OR MIT**, **MIT**, **MIT OR Unlicense**, **Zlib**, plus some **BSD-2/3-Clause** / **Unicode-3.0**. All are permissive and compatible with this repo’s Apache-2.0.

Rescan:

```powershell
cargo license --manifest-path native/Cargo.toml --avoid-dev-deps --avoid-build-deps
```

## 2. Guest (`guest/m2-async-smoke/`)

| Dependency | SPDX (scan) |
|------------|-------------|
| `wit-bindgen` (and wasm-tools related) | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT |

## 3. JVM / Android (Gradle)

| Dependency | Typical SPDX |
|------------|--------------|
| AndroidX (`core-ktx`, `appcompat`, test libs) | Apache-2.0 |
| `androidx.webgpu:webgpu:1.0.0-alpha05` | Dawn Java API + bundled `libwebgpu_c_bundled.so`. AndroidX artifact is Apache-2.0; Dawn native is typically BSD-3-Clause — see the AAR `NOTICE`. **Do not git-add the `.so`.** |
| Dawn C API (ND-DEFAULT product `.so`) | Same Dawn SHA as that AAR (`9d41fdf36977cca92361c6ae2769129bbaaafd9b`). Recipe [`scripts/build-dawn-c-android.py`](scripts/build-dawn-c-android.py) builds `libwebgpu_dawn.so` (`webgpu.h` exports). BSD-3-Clause: [`native/third_party/dawn-c/LICENSE`](native/third_party/dawn-c/LICENSE). **Do not git-add the `.so`.** From **0.1.1** the press job packs it into the `host-dawn` AAR. Default APK excludes androidx `libwebgpu_c_bundled.so`. |
| Kotlin stdlib / Gradle plugins | Apache-2.0 |
| JUnit 4 | EPL-1.0 |
| Vendored Host Kotlin (`:host-dawn` `…experimental.host` / `dawn` / `abicm`) | MIT. Copied from `wasi-webgpu-jvm-mvp` (copyright 焚日流光 2026). Full text: [`host-dawn/third_party/wasi-webgpu-jvm-mvp/LICENSE`](host-dawn/third_party/wasi-webgpu-jvm-mvp/LICENSE). Origin commit: [`ORIGIN.txt`](host-dawn/third_party/wasi-webgpu-jvm-mvp/ORIGIN.txt). |

JUnit is test-only and does not enter a published AAR runtime.

## 4. Vendored WIT (`third_party/wasi-webgpu/`, `third_party/wasi-gfx/`)

| Tree | License |
|------|---------|
| `wasi-webgpu` tag `v0.3.0-rc.2` (`wit/webgpu.wit`, `wit/imports.wit`) | W3C Community CLA. Text: [`third_party/wasi-webgpu/v0.3.0-rc.2/LICENSE.md`](third_party/wasi-webgpu/v0.3.0-rc.2/LICENSE.md). Origin: [`ORIGIN.txt`](third_party/wasi-webgpu/v0.3.0-rc.2/ORIGIN.txt). |
| `wasi-gfx` tag `v0.2.0` (`wit/surface.wit`, `wit/surface-webgpu.wit`) | Upstream tree at this tag has **no LICENSE file**. Origin: [`ORIGIN.txt`](third_party/wasi-gfx/v0.2.0/ORIGIN.txt). Do not refresh unless [`docs/scheme/rfc.md`](docs/scheme/rfc.md) changes the pin. |

Do not refresh the webgpu tree unless [`docs/scheme/guest-shape.md`](docs/scheme/guest-shape.md) changes the pin.

## 5. Redistribution

Binaries that include `libwasmtime_android_kt.so` must keep this repo’s `LICENSE` / `NOTICE` and follow upstream Wasmtime (Apache-2.0 WITH LLVM-exception) plus any other embedded dependencies.
