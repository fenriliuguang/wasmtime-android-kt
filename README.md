# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Component runtime**

**English** | [中文](README.zh.md)

An **upstream Wasmtime** embedding for Android (JNI / ART) that hosts [Component Model](https://component-model.bytecodealliance.org/) guests, including **true CM async**, with **canonical [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)** as the first proposal world.

This repository is meant to be a **citable Android host** on the Wasm component chain — not a UI toolkit, not a **rewritten** Dawn, and not a production WASI distro. The **default product/test artifact includes Dawn**; the core runtime AAR does not. See [`rfc-pluggable-gpu-backend.md`](docs/scheme/rfc-pluggable-gpu-backend.md).

Status: **experimental**. No compliant wasi:webgpu product claim. No default Maven Central publish.

## Current plan

| Priority | What |
|----------|------|
| **P0** | Canonical `wasi:webgpu@0.3.0-rc.2` guest shape + true CM async ([`guest-shape.md`](docs/scheme/guest-shape.md)) |
| **P1** | Ratified WASI 0.3 primitives/packages as guests need them |
| **P2** | Track upstream Wasmtime ([`wasmtime-tracking.md`](docs/scheme/wasmtime-tracking.md)) |

Success criteria (2026-08-17): [`rfc-ecosystem-contribution.md`](docs/scheme/rfc-ecosystem-contribution.md) — reproducible, citable, able to file upstream issues. **P0 is unchanged.**

## Quick start

```powershell
# 1. Cross-compile libwasmtime_android_kt.so → android/jniLibs/
.\scripts\build-native-android.ps1

# 2. Assemble smoke APK
.\gradlew.bat :smoke-app:assembleDebug

# 3. Device / emulator instruments (ABI must match)
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

Pinned versions: [`docs/build.md`](docs/build.md) (NDK `28.2.13676358`, Rust `1.97.1`, AGP `9.3.1`).

GPU-backed instruments need an **unpublished** host library. That dependency is **not** removed in this docs change — see [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md).

## Modules

| Path | Role |
|------|------|
| `runtime-api/` | Public Kotlin surface (no Android dependency) |
| `runtime-jni/` | `NativeLoader` / JNI |
| `android/` | AAR + `jniLibs` |
| `smoke-app/` | Minimal Activity + instrumented tests |
| `native/` | Rust cdylib (`wasmtime` 47.0.2 + JNI) |
| `scripts/build-native-android.ps1` | cargo-ndk pipeline |

## Docs

English is canonical ([`docs/LANGUAGE.md`](docs/LANGUAGE.md)). Chinese siblings use `.zh.md`.

| Doc | Notes |
|-----|--------|
| [Contributing](CONTRIBUTING.md) | PR / CI / hub freeze |
| [Scheme index](docs/scheme/README.md) | Living plan only |
| [Ecosystem RFC](docs/scheme/rfc-ecosystem-contribution.md) | **Accepted:** citable host; old L4 dropped |
| [Pluggable GPU backend](docs/scheme/rfc-pluggable-gpu-backend.md) | **Accepted:** Dawn default bundle; core without Dawn |
| [Long-term plan](docs/scheme/long-term-plan.md) | L0–L5 |
| [Guest shape](docs/scheme/guest-shape.md) | WIT acceptance rules (S-series) |
| [wasi:webgpu roadmap](docs/scheme/roadmap-wasi-webgpu.md) | P0 slices |
| [WASI 0.3 surface](docs/scheme/wasi-p3-surface.md) | Ratified P3 cuts |
| [Threading](docs/mapping/threading-android.md) | Android / Dawn / CM pump |
| [Build](docs/build.md) | NDK / cargo-ndk / Gradle |
| [Archive](docs/archive/README.md) | Historical dual-product docs — do not implement from these |

Slice progress: GitHub Project and [`changelog/unreleased/`](changelog/unreleased/). Do not add a README row per slice.

## License

**Apache License 2.0** — [`LICENSE`](LICENSE), [`NOTICE`](NOTICE).  
Third-party: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) (Wasmtime is Apache-2.0 WITH LLVM-exception).
