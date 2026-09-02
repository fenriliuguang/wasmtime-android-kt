# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Component runtime**

**English** | [中文](README.zh.md)

An **upstream Wasmtime** embedding for Android (JNI / ART) that hosts [Component Model](https://component-model.bytecodealliance.org/) guests, including **true CM async**, with **canonical [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)** as the first proposal world.

This repository is meant to be a **citable Android host** on the Wasm component chain — not a UI toolkit, not a **rewritten** Dawn, and not a production WASI distro. The **default product/test artifact includes Dawn**; the core runtime AAR does not. See [`rfc.md`](docs/scheme/rfc.md).

Status: **experimental `0.x`**. No compliant wasi:webgpu / CTS claim. Coordinate **`0.1.0`** (not pressed). Publishing: [`.github/workflows/publish.yml`](.github/workflows/publish.yml).

## Current plan

| Priority | What |
|----------|------|
| **Remaining** | Dawn C full bind → wasi-gfx size/resize → remaining pin input streams — [`remaining.md`](docs/agent/remaining.md) |
| **P2** | Wasmtime pin — **named** ([`wasmtime-p2.md`](docs/agent/wasmtime-p2.md)) |

Do **not** file upstream GitHub issues. Non-urgent (never auto): `context.unconfigure`, timestamped `frame-event`, Lost/Outdated `result`, multi-window.

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

## Demo

Pack a guest wasm, load it with this Android runtime (`android-webgpu` or a source composite), and present on a Surface: [wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples). This repository does **not** vendor the app. `:smoke-app` here is instruments, not that demo.

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
| [Scheme index](docs/scheme/README.md) | Living plan |
| [RFC](docs/scheme/rfc.md) | Product / GPU host / gfx loop |
| [Remaining](docs/agent/remaining.md) | Living close-out |
| [Guest shape](docs/scheme/guest-shape.md) | WIT acceptance rules |
| [P2 playbook](docs/agent/wasmtime-p2.md) | Named: Wasmtime pin |
| [Threading](docs/mapping/threading-android.md) | Android / Dawn / CM pump |
| [Build](docs/build.md) | NDK / cargo-ndk / Gradle |

Slice progress: GitHub Project and [`changelog/unreleased/`](changelog/unreleased/). Do not add a README row per slice.

## License

**Apache License 2.0** — [`LICENSE`](LICENSE), [`NOTICE`](NOTICE).
Third-party: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) (Wasmtime is Apache-2.0 WITH LLVM-exception).
