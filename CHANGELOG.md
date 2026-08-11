# Changelog

All notable planning and code changes for this experimental Android-first Wasm runtime.

## Unreleased

### Code — M5 slice: native artifact layout (2026-08-11)

- Document formal `android/jniLibs/{arm64-v8a,x86_64}/` layout ([`docs/mapping/artifacts.md`](docs/mapping/artifacts.md))
- `build-native-android.ps1` writes `build-info.json` (sha256 / sizes / toolchain)
- Add `scripts/verify-native-android.ps1` (`-RequireAll` for dual ABI)

### Code — M5 slice: stable error types (2026-08-11)

- Kotlin: `WasmtimeApiException` / `Compile` / `Link` / `Trap` under `WasmtimeException` + `Kind`
- JNI throws typed subclasses (`native/src/error.rs`)
- Docs: [`docs/mapping/errors.md`](docs/mapping/errors.md) (M3 page points here)
- Instrument: `ErrorKindInstrumentedTest` (invalid bytes → `WasmtimeCompileException`)

### Code — M4 Dawn render smoke (2026-08-11)

- Depend on Track A `host-webgpu` (Dawn) in smoke-app main APK (native `.so` packaging)
- L1 flat experimental host subset: adapter/device/queue/surface/clear/present (u32 reps)
- Fixture `fixtures/m4/render_smoke` (`run-clear`); instrument `DawnRenderSmokeInstrumentedTest`
- Gap list: [`docs/mapping/gap-m4-vs-cube.md`](docs/mapping/gap-m4-vs-cube.md); threading note in `threading-android.md`

### Code — M3 L1→L2 request-adapter (2026-08-11)

- Depend on Track A `host-api` + `abi-cm` via mavenLocal (`docs/build-track-a-deps.md`)
- L1 registers `experimental:webgpu-cm/host@0.8.0#request-adapter` → Kotlin → `AbiCmHostBindings` / `CpuWasiWebGpuHost`
- Fixture `fixtures/m3/request_adapter`; instrument `RequestAdapterInstrumentedTest` (non-zero u32 rep)
- Error subset: [`docs/mapping/errors-m3.md`](docs/mapping/errors-m3.md)

### Code — M2 true CM async gate (2026-08-11)

- Engine enables `wasm_component_model_async`; linker registers `get` via `func_wrap_concurrent`
- Host creates `FutureReader` (oneshot), completes with `42`; guest `fixtures/m2/async_get` observes via `run`
- JNI `nativeCallRunConcurrent` → `pollster::block_on(run_concurrent(call_concurrent))`
- Docs: [`docs/mapping/threading-m2-async.md`](docs/mapping/threading-m2-async.md); instrument `AsyncCmGetInstrumentedTest`

### Code — M1 sync CM closed (2026-08-11)

- Kotlin API: `Engine` / `Store` / `Component` / `Linker` / `Instance` + `WasmtimeException` + `HostU32U32ToU32`
- JNI: compile / instantiate / call `(u32)->u32` and `(u32,u32)->u32`; Kotlin host `add` callback; host `widget` resource (u32 rep) via `make-widget` / `echo-widget`
- Fixtures: `add_one`, `host_add`, `widget_echo`; instruments cover instantiate, host import, resource round-trip

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
