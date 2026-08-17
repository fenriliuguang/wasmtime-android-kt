# Blocked: unpublished GPU host library

**English** | [中文](blocked-gpu-host.zh.md)

**This page is a stop sign, not a build guide.**

Instrumented Android tests and `runtime-jni` still depend on Maven-local artifacts that are **not part of this repository**:

| Gradle alias | Coordinates (`gradle/libs.versions.toml`) |
|--------------|-------------------------------------------|
| `libs.wasi.webgpu.host.api` | `io.github.fenriliuguang.wasi.webgpu.experimental:host-api:0.1.0-experimental` |
| `libs.wasi.webgpu.host.webgpu` | `…:host-webgpu:0.1.0-experimental` (Dawn `.so` in the APK) |
| `libs.wasi.webgpu.abi.cm` | `…:abi-cm:0.1.0-experimental` |

Call sites (do **not** delete in a docs PR):

- `runtime-jni/build.gradle.kts` — `api(libs.wasi.webgpu.host.api)` / `abi-cm`
- `runtime-jni/…/ExperimentalWebGpuBridge.kt` — `WasiWebGpuHost` / `CpuWasiWebGpuHost`
- `smoke-app/build.gradle.kts` — `host-webgpu` + test `host-api`
- `native/src/cm.rs` — leftover `experimental:webgpu-cm/host@0.8.0` registration

`settings.gradle.kts` uses `mavenLocal()` so those artifacts can resolve after a **separate** unpublished publish step.

## Why this RFC did not remove them

The 2026-08-17 docs/plan change **cuts the dual-product story from living docs**. It does **not** replace Dawn or the CPU host. Doing that in the same change would break every WebGPU instrument test.

## Target (docs already accepted)

[`scheme/rfc-pluggable-gpu-backend.md`](scheme/rfc-pluggable-gpu-backend.md): L1 has no Dawn; `:host-dawn` + default bundle `:android-webgpu`; guest `request-adapter` → `none` when unwired.

These Maven-local artifacts should become **implementation details of `:host-dawn`**, not of `:runtime-jni`.

## Still blocked for the code PR

How Dawn bytes enter `:host-dawn` (pick one; do not mix silently):

1. **Vendor** the existing host + Dawn `.so` into this repo.  
2. **Publish** those coordinates and depend on them only from `:host-dawn`.  
3. **mavenLocal** only for `:host-dawn` / `smoke-app` until (1) or (2).

Until that PR, the **runtime** and the **GPU backend** are still coupled in `:runtime-jni`.
