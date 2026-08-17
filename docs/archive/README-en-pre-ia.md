# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Wasm runtime (Track B)**

[中文](README.md) | **English**

> **Status: short-term M0–M5 archived; long-term plan current.**  
> **2026-08-16:** Parallel tracks ended; this repo moves toward canonical `wasi:webgpu` shape → ZH [`docs/scheme/rfc-wasi-webgpu-canonical-shape.md`](docs/scheme/rfc-wasi-webgpu-canonical-shape.md).  
> **Current plan:** WASI 0.3 + **wasi:webgpu (P0, canonical WIT)** + track upstream Wasmtime → [`docs/scheme/long-term-plan.md`](docs/scheme/long-term-plan.md) (ZH).  
> Archive: [`docs/scheme/archive/m0-m5-thin-l1.md`](docs/scheme/archive/m0-m5-thin-l1.md).  
> Sister project (Track A): [`../wasi-webgpu-jvm-mvp`](../wasi-webgpu-jvm-mvp) — **simple demo** (experimental cube + wasmtime4j + locked sync-compat).  
> Full charter: [`docs/scheme/charter.en.md`](docs/scheme/charter.en.md) / ZH [`charter.md`](docs/scheme/charter.md).  
> **Build guide:** [`docs/build.md`](docs/build.md) (Chinese; commands are OS-agnostic).

## One-liner

**Long-term:** Android-first Java/Kotlin Component runtime — prioritize **ratified WASI 0.3**, proposal focus **canonical wasi:webgpu**, engine = **upstream Wasmtime** only.  
**Validated base (archived):** thin JNI L1 can call Track A Dawn Host as a backend, with true CM async.  
**Track A** is a demo only; this repo is the sole place advancing wasi:webgpu guest shape.

## Track A vs this repo

| Track | Repo | Runtime | Async | Role |
|-------|------|---------|-------|------|
| **A** | `wasi-webgpu-jvm-mvp` | wasmtime4j + patches | **locked sync-compat** | **Simple demo** (experimental cube) |
| **B** | **this repo** | upstream Wasmtime + custom JNI | true CM async | Android-first; **owns** wasi:webgpu WIT shape |

Hard rule: **do not** reimplement Dawn; **do not** silently replace Track A’s default runtime; **do not** advance the same guest ABI in parallel.

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
| [Contributing](CONTRIBUTING.md) | PR / CI / hub-freeze (ZH) |
| [Plan-change RFC](docs/scheme/rfc-wasi-webgpu-canonical-shape.md) | **Accepted** (ZH): end parallel tracks; canonical shape; S-series |
| [Long-term plan](docs/scheme/long-term-plan.md) | **Current** (ZH): WASI 0.3 · wasi:webgpu · Wasmtime |
| [Scheme index](docs/scheme/README.en.md) | Stage table |
| [Charter](docs/scheme/charter.en.md) | Vision / risks (EN may lag ZH) |
| [M0–M5 archive](docs/scheme/archive/m0-m5-thin-l1.md) | Short-term thin L1 close-out |
| [Milestones (historical)](docs/scheme/milestones.en.md) | Frozen M0–M5 DoD |
| [Build](docs/build.md) | Repro steps |
| [Dual-track / boundary](docs/scheme/dual-track.en.md) | Track A = demo; this repo owns shape |
| [Changelog](CHANGELOG.md) | Rolled-up history; in-flight: [`changelog/unreleased/`](changelog/unreleased/) |

> Do **not** add a row here for every short PR. Slice docs belong on topic pages (`wasi-p3-surface`, `roadmap-wasi-webgpu`, scheme index).

## Delivered now

- **Plan change (2026-08-16):** Track A is demo-only; this repo moves toward canonical wasi:webgpu  
- **Archived** M0–M5 thin L1 base  
- **No** wasmtime4j dependency; **no** silent replace of Track A’s default Demo  
- Slice progress lives on the RFC / roadmap and `changelog/unreleased/`, not this page

## License

**Apache License 2.0** — see [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE).  
Third-party summary: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) (Wasmtime is Apache-2.0 WITH LLVM-exception).
