# wasmtime-android-kt

**experimental · Android-first Java/Kotlin Component runtime**

**English** | [中文](README.zh.md)

An **upstream Wasmtime** embedding for Android (JNI / ART) that hosts [Component Model](https://component-model.bytecodealliance.org/) guests, including **true CM async**, with **canonical [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu)** as the first proposal world.

This repository is a **citable Android host** on the Wasm component chain — not a UI toolkit, not a **rewritten** Dawn, and not a production WASI distro. The **default product/test artifact includes Dawn**; the core runtime AAR does not. See [`rfc.md`](docs/scheme/rfc.md).

Status: **experimental `0.x`**. Coordinate **`0.1.0`** (pressed). No compliant wasi:webgpu / CTS claim. Product subset: [`claim-010.md`](docs/scheme/claim-010.md). Later publishes: [`.github/workflows/publish.yml`](.github/workflows/publish.yml) (tag `v*` on `main` or `workflow_dispatch` from `main`, GitHub Environment `release`).

Do **not** file upstream GitHub issues. Non-urgent: `context.unconfigure`, timestamped `frame-event`, Lost/Outdated `result`, multi-window.

## Use `0.1.0`

minSdk **24**. Repositories: `mavenCentral()` + `google()` (`androidx.webgpu`). R8/minify must keep the AAR `consumer-rules.pro`. Sockets / outbound HTTP need the Android **INTERNET** permission.

Recommended (0.x default bundle — runtime + Dawn host):

```kotlin
dependencies {
    implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:0.1.0")
}
```

BYO / no GPU: `…:runtime:0.1.0`. Dawn host only: `…:host-dawn:0.1.0`. Do **not** depend on `runtime-api` / `runtime-jni` directly (Maven transitives of `runtime`). Never depend on `:smoke-app`.

Source checkout / `includeBuild` still works if you are not consuming Maven.

### Host

Public SPI: `Engine` / `Store` / `Linker` / `Component` / `Instance`, `GpuBackends.dawn()`, `Store.setWebGpuBackend`. Compile, instantiate, and `callRunConcurrent` on a dedicated **GpuThread** — not the ART main thread ([threading](docs/mapping/threading-android.md)).

```kotlin
Engine.create().use { engine ->
    Component.compile(engine, wasmBytes).use { component ->
        Linker.create(engine).use { linker ->
            Store.create(engine).use { store ->
                store.setWebGpuBackend(GpuBackends.dawn())
                store.bindCanvasNativeWindow(
                    NativeBridge.nativeWindowFromSurface(surface),
                    width,
                    height,
                )
                linker.instantiate(store, component).use { instance ->
                    // Choreographer / GpuThread: store.postGfxVsync(frameTimeNanos)
                    val frames = instance.callRunConcurrent(store)
                    store.closeGfxOnFrame()
                }
            }
        }
    }
}
```

- No backend → guest `gpu.request-adapter` returns **`none`**. `Store.createWithDiscoveredBackend` is ServiceLoader convenience; **`setWebGpuBackend` always wins**.
- Product `Linker.create` omits fixture constructors (`get-device`, HTTP request/response ctors). Pin `get-gpu` stays.
- Present loop: guest **pulls** `wasi-gfx:surface@0.2.0` `on-frame`; host posts vsync with `Store.postGfxVsync`. `surfaceDestroyed` → `closeGfxOnFrame`. Pointer / key: `postGfxPointer` / `postGfxKey`.
- Close `Engine` / `Store` / `Linker` / `Component` / `Instance` (all `AutoCloseable`).

### Guest

Ship a **Component** wasm (not core-module only). Pin **`wasi:webgpu@0.3.0-rc.2`**: chain `get-gpu` → `request-adapter` → `request-device`. Continuous on-screen: **`wasi-gfx:surface@0.2.0`**. WIT rules: [`guest-shape.md`](docs/scheme/guest-shape.md). What `0.1.0` actually covers: [`claim-010.md`](docs/scheme/claim-010.md).

End-to-end app (pack a guest, load, present): [wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples). This repository does **not** vendor that app. `:smoke-app` here is instruments, not the demo.

## Build from source

```powershell
# 1. Cross-compile libwasmtime_android_kt.so → android/jniLibs/
.\scripts\build-native-android.ps1

# 2. Assemble smoke APK
.\gradlew.bat :smoke-app:assembleDebug

# 3. Device / emulator instruments (ABI must match)
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

NDK `28.2.13676358`, Rust `1.97.1`, AGP `9.3.1` — [`tech-stack.md`](docs/scheme/tech-stack.md). Workflow: [`CONTRIBUTING.md`](CONTRIBUTING.md). Rebuild natives with the same script. Out-of-tree examples gate (includeBuild, no mavenLocal): `.\scripts\verify-examples-gate.ps1`.

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
| [Contributing](CONTRIBUTING.md) | PR / CI / publish |
| [Scheme index](docs/scheme/README.md) | RFC and shape docs |
| [RFC](docs/scheme/rfc.md) | Product / GPU host / gfx loop |
| [Guest shape](docs/scheme/guest-shape.md) | WIT acceptance rules |
| [Claim table](docs/scheme/claim-010.md) | `0.1.0` product subset |
| [Threading](docs/mapping/threading-android.md) | Android / Dawn / CM pump |

Slice progress: GitHub Project and [`changelog/unreleased/`](changelog/unreleased/). Do not add a README row per slice.

## License

**Apache License 2.0** — [`LICENSE`](LICENSE), [`NOTICE`](NOTICE).
Third-party: [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) (Wasmtime is Apache-2.0 WITH LLVM-exception).
