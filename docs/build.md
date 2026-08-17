# How to build

**English** | [中文](build.zh.md)

Pins must stay in sync with [`scheme/tech-stack.md`](scheme/tech-stack.md) and a changelog fragment.

## Prerequisites

| Tool | Pinned version |
|------|----------------|
| JDK | 17+ (Gradle Daemon via Foojay uses **21**) |
| Android SDK | includes `platforms;android-36` |
| NDK | **28.2.13676358** |
| Rust | **1.97.1** (`native/rust-toolchain.toml`) |
| cargo-ndk | `cargo install cargo-ndk` |
| Gradle | Wrapper **9.6.1** (in-repo) |

SDK env (either): `ANDROID_SDK_ROOT` / `ANDROID_HOME`.  
Or gitignored `local.properties` at the repo root:

```properties
sdk.dir=C\:\\Users\\<you>\\AppData\\Local\\Android\\Sdk
```

```powershell
sdkmanager --install "ndk;28.2.13676358"
rustup toolchain install 1.97.1
cargo install cargo-ndk
```

GPU-backed instruments use in-tree `:host-dawn` plus published `androidx.webgpu` — [`blocked-gpu-host.md`](blocked-gpu-host.md). That is **not** required to compile `:runtime-api` or `:runtime-jni`, or to build the native Wasmtime `.so`.

## 1. Android `.so`

Official layout: [`mapping/artifacts.md`](mapping/artifacts.md).

```powershell
.\scripts\build-native-android.ps1
```

Default (dual ABI + metadata):

```text
android/jniLibs/arm64-v8a/libwasmtime_android_kt.so
android/jniLibs/x86_64/libwasmtime_android_kt.so
android/jniLibs/build-info.json
```

arm64 only:

```powershell
.\scripts\build-native-android.ps1 -Targets arm64-v8a
```

Verify (no compile; `-RequireAll` wants both ABIs):

```powershell
.\scripts\verify-native-android.ps1 -RequireAll
```

Notes:

- `JNI_OnLoad` returns **`JNI_VERSION_1_6`** (ART rejects 1_8).  
- Bionic has no `libpthread`: scripts use `native/link-stubs/libpthread.so` → `INPUT(-lc)`.  
- Windows cross-compile defaults `CARGO_PROFILE_RELEASE_OPT_LEVEL=2` (keeps `stream.write` / cli stdio instrument frames smaller), then `llvm-strip`. If rustc `ACCESS_VIOLATION`, set `$env:CARGO_PROFILE_RELEASE_OPT_LEVEL="0"` and rebuild. That variable overrides Cargo **release** `opt-level` (`0` = no opt, large frames; `1`/`2`/`3`/`s`/`z` — [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#opt-level)).  
- After a successful build: write `build-info.json` and verify the `-Targets` of that run.

## 2. JVM / Android modules

Need step 1 first (otherwise `smoke-app` instruments lack `.so`).

```powershell
.\gradlew.bat :runtime-api:compileKotlin :runtime-jni:compileKotlin :android:assembleDebug :smoke-app:assembleDebug
```

`:host-dawn` / `:android-webgpu` / `:smoke-app` pull Dawn via `androidx.webgpu` — see [`blocked-gpu-host.md`](blocked-gpu-host.md). `:runtime-jni` does not.

## 3. Device / emulator instruments

Device ABI must match the produced `.so` (physical devices prefer **arm64-v8a**).

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
```

Includes `LoadLibraryInstrumentedTest` — `loadLibrary` + non-empty `nativeWasmtimeVersion`.

OEM / UTP races: `adb shell am instrument` is a known workaround.

## Modules

| Module | Role |
|--------|------|
| `runtime-api` | Public SPI (`WebGpuBackend`) / future Engine API (no Android dependency) |
| `runtime-jni` | `NativeLoader` / JNI / L1 (no Dawn) |
| `android` | AAR + Wasmtime `jniLibs` |
| `host-dawn` | Dawn adapter + vendored Host Kotlin + `androidx.webgpu` `.so` |
| `android-webgpu` | Product default bundle (`api(android)` + `api(host-dawn)`) |
| `smoke-app` | Minimal Activity + instruments (depends on the bundle) |
| `native/` | Rust cdylib (not a Gradle subproject) |

## 4. Optional desktop shell

No NDK: host cdylib + JVM smoke (**not** the formal gate):

```powershell
.\scripts\build-native-host.ps1
.\gradlew.bat :runtime-jni:test
```

Full contributor flow: [`contribute.md`](contribute.md).

## Explicitly out of scope

- **No** wasmtime4j / 4j native as the runtime  
- Desktop shell **does not** replace Android instruments  
