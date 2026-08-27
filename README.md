# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Component runtime**

**English** | [中文](README.zh.md)

An **upstream Wasmtime** embedding for Android (JNI / ART) that hosts [Component Model](https://component-model.bytecodealliance.org/) guests, including **true CM async**, with **canonical [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)** as the first proposal world.

This repository is meant to be a **citable Android host** on the Wasm component chain — not a UI toolkit, not a **rewritten** Dawn, and not a production WASI distro. The **default product/test artifact includes Dawn**; the core runtime AAR does not. See [`rfc-pluggable-gpu-backend.md`](docs/scheme/rfc-pluggable-gpu-backend.md).

Status: **experimental `0.x`**. No compliant wasi:webgpu / CTS claim. Coordinate **`0.1.0`**. Publishing: [`.github/workflows/publish.yml`](.github/workflows/publish.yml) ([`rfc-l5-productization.md`](docs/scheme/rfc-l5-productization.md)).

## Current plan

| Priority | What |
|----------|------|
| **P0** | Canonical `wasi:webgpu@0.3.0-rc.2` guest shape + true CM async — **closed** 2026-08-22 ([`guest-shape.md`](docs/scheme/guest-shape.md)) |
| **P1** | Ratified WASI 0.3 official package shapes + device instruments — **closed** 2026-08-26 ([`p1-wasi-p3.md`](docs/archive/p1-wasi-p3.md)) |
| **P2** | Wasmtime pin — **named** ([`wasmtime-p2.md`](docs/agent/wasmtime-p2.md), [`wasmtime-tracking.md`](docs/scheme/wasmtime-tracking.md)) |
| **L5 / 0.1.0** | Product subset + gfx loop + **`0.1.0` coordinates / publish workflow** ([`product-010.md`](docs/agent/product-010.md)) |

Success criteria (2026-08-17, amended 2026-08-21): [`rfc-ecosystem-contribution.md`](docs/scheme/rfc-ecosystem-contribution.md) — reproducible, citable; **do not** file upstream GitHub issues. **P0 and P1 are closed.** **`0.1.0` product queue is empty** after P010-PUB. P2 Wasmtime pin is **named**.

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

## Consume `0.1.0`

Recommended (0.x default bundle — runtime + Dawn host):

```kotlin
dependencies {
    implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:0.1.0")
}
```

BYO / no GPU: `…:runtime:0.1.0`. Dawn host only: `…:host-dawn:0.1.0`. Do **not** depend on `runtime-api` / `runtime-jni` directly (Maven transitives of `runtime`). Never depend on `:smoke-app`.

Artifacts appear on Maven Central / GitHub Packages after a maintainer runs [`.github/workflows/publish.yml`](.github/workflows/publish.yml) with arm64 `android/jniLibs/` and (for Central) Portal + GPG secrets. Until that press, use this repo as a source checkout. Minify must consume the AAR `consumer-rules.pro`. Rebuild natives: [`scripts/build-native-android.ps1`](scripts/build-native-android.ps1).

GPU-backed instruments use in-tree `:host-dawn` plus published `androidx.webgpu` — [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md).

## Modules

| Path | Role | Maven artifactId |
|------|------|------------------|
| `runtime-api/` | Public Kotlin surface (no Android dependency) | `runtime-api` (transitive only) |
| `runtime-jni/` | `NativeLoader` / JNI | `runtime-jni` (transitive only) |
| `android/` | AAR + `jniLibs` | **`runtime`** |
| `host-dawn/` | Dawn / androidx.webgpu backend | `host-dawn` |
| `android-webgpu/` | Default bundle (`api` of runtime + host-dawn) | **`android-webgpu`** |
| `smoke-app/` | Minimal Activity + instrumented tests | **not published** |
| `native/` | Rust cdylib (`wasmtime` 47.x + JNI) | — |
| `scripts/build-native-android.ps1` | cargo-ndk pipeline | — |

## Docs

English is canonical ([`docs/LANGUAGE.md`](docs/LANGUAGE.md)). Chinese siblings use `.zh.md`.

| Doc | Notes |
|-----|--------|
| [Contributing](CONTRIBUTING.md) | PR / CI / hub freeze |
| [Scheme index](docs/scheme/README.md) | Living plan only |
| [Ecosystem RFC](docs/scheme/rfc-ecosystem-contribution.md) | **Accepted:** citable host; old L4 dropped |
| [Pluggable GPU backend](docs/scheme/rfc-pluggable-gpu-backend.md) | **Accepted:** Dawn default bundle; core without Dawn |
| [L5 productization](docs/scheme/rfc-l5-productization.md) | **Accepted:** 0.x class B; Central at `0.1.0` |
| [wasi-gfx frame loop](docs/scheme/rfc-wasi-gfx-frame-loop.md) | **Accepted intent:** `0.1.0` present loop (not P0) |
| [Long-term plan](docs/scheme/long-term-plan.md) | L0–L5 |
| [Guest shape](docs/scheme/guest-shape.md) | WIT acceptance rules (S-series) |
| [wasi:webgpu roadmap](docs/scheme/roadmap-wasi-webgpu.md) | P0 slices |
| [WASI 0.3 surface](docs/scheme/wasi-p3-surface.md) | P1 cuts (archived; stub → [`p1-wasi-p3-surface.md`](docs/archive/p1-wasi-p3-surface.md)) |
| [P2 playbook](docs/agent/wasmtime-p2.md) | Named: Wasmtime pin |
| [0.1.0 playbook](docs/agent/product-010.md) | L5 product gates (**queue empty** after P010-PUB) |
| [Threading](docs/mapping/threading-android.md) | Android / Dawn / CM pump |
| [Build](docs/build.md) | NDK / cargo-ndk / Gradle |
| [Archive](docs/archive/README.md) | Historical dual-product docs — do not implement from these |

Slice progress: GitHub Project and [`changelog/unreleased/`](changelog/unreleased/). Do not add a README row per slice.

## License

**Apache License 2.0** — [`LICENSE`](LICENSE), [`NOTICE`](NOTICE).  
Third-party: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) (Wasmtime is Apache-2.0 WITH LLVM-exception).
