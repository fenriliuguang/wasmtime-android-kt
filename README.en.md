# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Wasm runtime (Track B)**

[中文](README.md) | **English**

> **Status: short-term M0–M5 archived; long-term plan chartered (2026-08-11, docs-only).**  
> **Current plan:** WASI 0.3 + **wasi:webgpu (P0)** + track upstream Wasmtime → [`docs/scheme/long-term-plan.md`](docs/scheme/long-term-plan.md) (ZH).  
> Archive: [`docs/scheme/archive/m0-m5-thin-l1.md`](docs/scheme/archive/m0-m5-thin-l1.md).  
> Sister project (Track A): [`../wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) — **locked sync-compat** + wasmtime4j; device acceptance remains CM cube.  
> Full charter: [`docs/scheme/charter.en.md`](docs/scheme/charter.en.md) / ZH [`charter.md`](docs/scheme/charter.md).  
> **Build guide:** [`docs/build.md`](docs/build.md) (Chinese; commands are OS-agnostic).

## One-liner

**Long-term:** Android-first Java/Kotlin Component runtime — prioritize **ratified WASI 0.3**, proposal focus **wasi:webgpu**, engine = **upstream Wasmtime** only.  
**Validated base (archived):** thin JNI L1 plugs into Track A L2 (Dawn) with true CM async.

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
| [Long-term plan](docs/scheme/long-term-plan.md) | **Current** (ZH): WASI 0.3 · wasi:webgpu · Wasmtime |
| [Scheme index](docs/scheme/README.en.md) | Stage table |
| [Charter](docs/scheme/charter.en.md) | Vision / risks (EN may lag ZH) |
| [M0–M5 archive](docs/scheme/archive/m0-m5-thin-l1.md) | Short-term thin L1 close-out |
| [Milestones (historical)](docs/scheme/milestones.en.md) | Frozen M0–M5 DoD |
| [Build](docs/build.md) | Repro steps |
| [Dual-track](docs/scheme/dual-track.en.md) | Contract with Track A |
| [Changelog](CHANGELOG.md) | History |

## Delivered now

- **Long-term planning docs** (no new code required in this tranche)
- **Archived** M0–M5 thin L1 base (sync CM, true CM async, experimental webgpu→L2, Dawn smoke)
- **No** wasmtime4j dependency; **no** silent replace of Track A acceptance
