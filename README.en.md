# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Wasm runtime (Track B)**

[中文](README.md) | **English**

> **Status: M0 build skeleton landed (2026-08-10).** No Component Model runtime API yet (M1+).  
> Sister project (Track A): [`../wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) — **locked sync-compat** + wasmtime4j; device acceptance remains CM cube.  
> Full charter: [`docs/scheme/charter.en.md`](docs/scheme/charter.en.md).  
> **Build guide:** [`docs/build.md`](docs/build.md) (Chinese; commands are OS-agnostic).

## One-liner

**Long-term:** a Java/Kotlin Wasm runtime optimized for Android (Component Model first).  
**Near-term:** a thin L1 over **upstream Wasmtime** (JNI) that plugs into Track A’s existing L2 (`WasiWebGpuHost` / Dawn) and can host true CM async.

## Dual-track

| Track | Repo | Runtime | Async | Role |
|-------|------|---------|-------|------|
| **A** | `wasi-webgpu-jvm-mvp` | wasmtime4j + in-repo patches | **locked sync-compat** | Demo / CI / CM cube acceptance |
| **B** | **this repo** `wasmtime-android-kt` | upstream Wasmtime + custom JNI | true CM async (goal) | Android-first; does not block A |

Hard rule: **L2 must not depend on L1**; share Host/ABI constants; isolate natives and CI gates.

## Quick start (M0)

```powershell
.\scripts\build-native-android.ps1
.\gradlew.bat :smoke-app:assembleDebug
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

Pinned versions: NDK `28.2.13676358`, Rust `1.97.1`, AGP `9.3.1` — see [`docs/build.md`](docs/build.md).

## Modules

| Path | Role |
|------|------|
| `runtime-api/` | Public Kotlin surface (no Android deps) |
| `runtime-jni/` | `NativeLoader` / JNI decls |
| `android/` | AAR + `jniLibs` |
| `smoke-app/` | Minimal Activity + `LoadLibraryInstrumentedTest` |
| `native/` | Rust cdylib (`wasmtime` 47.0.2 + JNI) |
| `scripts/build-native-android.ps1` | cargo-ndk pipeline |

## Docs

| Doc | Notes |
|-----|--------|
| [Charter](docs/scheme/charter.en.md) | Vision, stack, risks |
| [Build](docs/build.md) | Repro steps |
| [Milestones](docs/scheme/milestones.en.md) | M0–M5 DoD |
| [Dual-track](docs/scheme/dual-track.en.md) | Contract with Track A |
| [Changelog](CHANGELOG.md) | History |

## Delivered now

- Planning docs + **M0 Gradle / native skeleton**
- `JNI_OnLoad` → `JNI_VERSION_1_6`; instrumented `loadLibrary` test
- **No** CM instantiate / async / L2 wiring (M1+)
- **No** wasmtime4j dependency
