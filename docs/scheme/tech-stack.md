# Tech stack

**English** | [中文](tech-stack.zh.md)

## Engine

| Item | Choice |
|------|--------|
| Crate | Official `wasmtime` **47.x** (currently `47.0.4`) |
| Features | `component-model` + `component-model-async` + required `async` |
| Forbidden | wasmtime4j native / `dlopen` of a 4j `.so` |
| Tracking | [`wasmtime-tracking.md`](wasmtime-tracking.md) |

## Binding

Rust `cdylib` → JNI (`jni` crate) → thin Kotlin/Java API. `JNI_OnLoad` must return an ART-safe version (`JNI_VERSION_1_6`). No Panama on the Android path.

Guest marshalling: [`guest-shape.md`](guest-shape.md). Backend GPU handles may be `u32` reps; guests must not see bare `u32` as the product return shape.

## Android

| Item | Choice |
|------|--------|
| ABI | `arm64-v8a` primary, `x86_64` emulator secondary |
| NDK | `28.2.13676358` |
| AGP | `9.3.1` |
| minSdk | 24 (smoke-app) |

Pointers: treat as unsigned where TBI/PAC apply.

## GPU backend

Target: [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md).

| Artifact | Dawn `.so` |
|----------|------------|
| `:android` (L1) | **No** |
| `:host-dawn` | **Yes** (adapter + native) |
| `:android-webgpu` (product default) | Yes, via `host-dawn` |

SPI lives in `runtime-api`. Do not leak foreign `WasiWebGpuHost` types into L1. Canonical guests: `wasi:webgpu@0.3.0-rc.2`. Do not add new `experimental:webgpu-cm` exports.

**Today:** Host Kotlin lives in `:host-dawn`; Dawn `.so` is `androidx.webgpu` (not git). [`../blocked-gpu-host.md`](../blocked-gpu-host.md). `:runtime-jni` does not depend on Dawn types.

## Build

Gradle + cargo-ndk. [`../build.md`](../build.md). Optional desktop cdylib: [`../contribute.md`](../contribute.md).
