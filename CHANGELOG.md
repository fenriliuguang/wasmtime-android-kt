# Changelog

All notable planning and code changes for this experimental Android-first Wasm runtime.

## Unreleased

### Code — M1 slice 1 sync CM instantiate (2026-08-11)

- Kotlin API: `Engine` / `Store` / `Component` / `Linker` / `Instance` + `WasmtimeException`
- JNI: compile component bytes, empty-linker instantiate, call root `(u32)->u32` export
- Fixture: `fixtures/m1/add_one.wasm` (`run` → `a + 1`); instrumented `SyncCmInstantiateInstrumentedTest`

### Code — M0 device gate (2026-08-11)

- Fix smoke-app androidTest deps: add Espresso (brings `AndroidJUnitRunner`) and use `androidTestUtil` for `test-services` (align Track A; OEM/UTP)
- Verified on device (arm64-v8a): `LoadLibraryInstrumentedTest.loadLibraryAndQueryWasmtimeVersion` green

### Code — M0 build skeleton (2026-08-10)

- Gradle multi-module: `runtime-api`, `runtime-jni`, `android`, `smoke-app` (AGP 9.3.1 / Kotlin 2.1.21 / Gradle 9.6.1)
- Rust cdylib `native/` → `libwasmtime_android_kt.so` over upstream `wasmtime` **47.0.2** (CM + async features)
- `JNI_OnLoad` returns **`JNI_VERSION_1_6`** (ART-safe); M0 probes `nativeRuntimeId` / `nativeWasmtimeVersion`
- `scripts/build-native-android.ps1` (NDK 28.2.13676358, cargo-ndk, pthread stub, Windows opt-level workaround)
- Docs: [`docs/build.md`](docs/build.md); package root `io.github.fenriliuguang.wasmtime.android`

### Planning — repository chartered (docs only, 2026-08-10)

- Create `wasmtime-android-kt` as **Track B** beside `wasi-webgpu-jvm-mvp` (Track A)
- Long-term vision: Java/Kotlin Wasm runtime specialized for Android
- Near-term: thin L1 over **upstream Wasmtime** (JNI), true CM async capable; plug into Track A L2
- Docs: charter, dual-track contract, tech-stack, milestones M0–M5, non-goals, Android threading draft (ZH + EN)
- Track A remains **locked sync-compat** (see sister repo)
