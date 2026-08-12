# Changelog

All notable planning and code changes for this experimental Android-first Wasm runtime.

## Unreleased

### Code — WASI 0.3 wasi:cli stderr write-via-stream smoke (2026-08-12)

- Register `wasi:cli/stderr@0.3.0#write-via-stream` (shared `pipe_stream_byte_count` with stdout / root `take`)
- **Transitional** signature: `stream<u8> -> future<u32>` byte count (not official `future<result<_, error-code>>`)
- Fixture `fixtures/wasi/cli_stderr` (`ERR\n`); native `wasi_cli_stderr`; instrument `WasiCliStderrInstrumentedTest`
- CI includes `--test wasi_cli_stderr`; stdin / `wasi:cli/command` deferred

### Code — WASI 0.3 wasi:clocks monotonic-clock.wait-until smoke (2026-08-12)

- Register `wasi:clocks/monotonic-clock@0.3.0#wait-until` (`func_wrap_concurrent` + oneshot / helper-thread sleep; same Instant epoch as `#now`; keep `#now` + `#wait-for`)
- Fixture `fixtures/wasi/monotonic_wait_until`; native `wasi_monotonic_wait_until`; instrument `WasiMonotonicWaitUntilInstrumentedTest` via `callRunConcurrent`
- CI includes `--test wasi_monotonic_wait_until`; `system-clock` / timezone deferred

### Code — W1 wasi:webgpu dual-register request-adapter (2026-08-12)

- Dual-register transitional flat `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter` onto the same L2 sync u32 path as experimental (not `[method]gpu.request-adapter`; not true async)
- Fixture `fixtures/w1/webgpu_request_adapter`; native `wasi_webgpu_request_adapter`; twin instrument `WasiWebGpuRequestAdapterInstrumentedTest`; `copyW1Fixtures`
- Re-pin proposal tag `v0.3.0-rc.2` → `6a776bada0b66d3dbf9da304a49ff2947ce4e1f8`; mark W1 delivered in scheme docs; CI `--test wasi_webgpu_request_adapter`

### Fix — Android stream write / cli stdout instrumented stack overflow (2026-08-12)

- Empty-chunk `StreamConsumer` returns `Pending`+wake (avoid Completed sync-reentry)
- `nativeCallStreamWrite` drives via `run_concurrent` / `call_concurrent` (same pump as M2)
- Fix `pipe_stream_byte_count` store reborrow (`&mut *store`) after conflict merge
- Note: Windows default `CARGO_PROFILE_RELEASE_OPT_LEVEL=0` still deepens frames; device stream smoke validated with `=2`

### Code — WASI 0.3 wasi:cli stdout write-via-stream smoke (2026-08-12)

- Register `wasi:cli/stdout@0.3.0#write-via-stream` (shared `CollectConsumer` / `pipe` with root `take`)
- **Transitional** signature: `stream<u8> -> future<u32>` byte count (not official `future<result<_, error-code>>`)
- Fixture `fixtures/wasi/cli_stdout` (`OUT\n`); native `wasi_cli_stdout`; instrument `WasiCliStdoutInstrumentedTest`
- CI includes `--test wasi_cli_stdout`; stdin / `wasi:cli/command` deferred (stderr: see Unreleased stderr entry)
### Code — WASI 0.3 wasi:clocks monotonic-clock.wait-for smoke (2026-08-12)

- Register `wasi:clocks/monotonic-clock@0.3.0#wait-for` (`func_wrap_concurrent` + oneshot / helper-thread sleep; keep `#now`)
- Fixture `fixtures/wasi/monotonic_wait_for`; native `wasi_monotonic_wait_for`; instrument `WasiMonotonicWaitForInstrumentedTest` via `callRunConcurrent`
- CI includes `--test wasi_monotonic_wait_for`; `wait-until` / `system-clock` / timezone deferred

### Code — WASI 0.3 wasi:clocks monotonic-clock.now smoke (2026-08-12)

- Register `wasi:clocks/monotonic-clock@0.3.0#now` (process-wide Instant epoch → non-decreasing mark)
- Fixture `fixtures/wasi/monotonic_now`; native `wasi_monotonic_now`; instrument `WasiMonotonicClockInstrumentedTest`
- CI includes `--test wasi_monotonic_now`; wait-* / system-clock / timezone deferred
### Docs — W1 dual-register cut plan (2026-08-12)

- Add [`docs/scheme/w1-dual-register.md`](docs/scheme/w1-dual-register.md): minimal DoD for dual-registering `wasi:webgpu` names onto existing L2 sync `request-adapter`; pin restated; guest/instrument/out-of-scope
- Link from gap §5, roadmap W1, and [`vcs-workflow.md`](docs/scheme/vcs-workflow.md) §7（next code knife `feat/webgpu-w1-…`）

### Code — WASI 0.3 wasi:random get-random-u64 smoke (2026-08-12)

- Register `wasi:random/random@0.3.0#get-random-u64` (host CSPRNG via `getrandom`)
- Fixture `fixtures/wasi/random_u64`; native `wasi_random_u64`; instrument `WasiRandomInstrumentedTest`
- JNI/Kotlin `nativeCallUnitToU64` / `Instance.callUnitToU64`; CI includes the new cargo test

### Code — P3-PRIM-5 stream write / host consume smoke (2026-08-12)

- Guest `stream.new` + `stream.write` (`P3WR`) → host `take` pipes `StreamConsumer`, returns `future<u32>` byte count (`fixtures/p3/stream_write`)
- Native test `p3_stream_write`; JNI `nativeCallStreamWrite` / `Instance.callStreamWrite`
- Instrument `StreamWriteInstrumentedTest`; docs: wasi-p3-surface + threading-m2-async
- Fix duplicate `nativeCallStreamRead` JNI export in `cm.rs`

### Chore — Apache-2.0 license + third-party notices (2026-08-11)

- Add root [`LICENSE`](LICENSE) (Apache-2.0), [`NOTICE`](NOTICE), [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
- Align with `native/Cargo.toml` `license = "Apache-2.0"`; set same on guest crate; link from README (ZH+EN)
### Fix — CI native job OOM / exit 101 (2026-08-11)

- Cap `CARGO_BUILD_JOBS=2`, strip debuginfo in CI; install JDK + build-essential
- Run locked integration tests `m2_async_get` + `p3_stream_read` (avoid full `--all-targets` memory spike)

### Chore — OSS PR readiness (CI, CONTRIBUTING, templates) (2026-08-11)

- Add `.github/workflows/ci.yml`: `cargo test` in `native/` + `:runtime-api:compileKotlin`; aggregate check name `CI` for Rulesets
- Add [`CONTRIBUTING.md`](CONTRIBUTING.md) (workflow, CI, Write vs Fork permissions)
- Add PR / Issue templates under `.github/`
- Update [`vcs-workflow.md`](docs/scheme/vcs-workflow.md) open-source checklist

### Code — P3-PRIM-3 stream read smoke (2026-08-11)

- Host `StreamReader::new` with fixed `P3ST` bytes → guest canon `stream.read` (`fixtures/p3/stream_read`)
- Native test `p3_stream_read`; JNI `nativeCallStreamRead` / `Instance.callStreamRead`
- Instrument `StreamReadInstrumentedTest` (packed result 65); docs: wasi-p3-surface + threading-m2-async

### Docs — W0 gap: experimental vs wasi:webgpu (2026-08-11)

- Add [`docs/mapping/gap-experimental-vs-wasi-webgpu.md`](docs/mapping/gap-experimental-vs-wasi-webgpu.md): map 13 flat experimental imports to `wasi:webgpu@0.3.0-rc.2`; pin proposal; W1–W4 next cuts
- Link from [`roadmap-wasi-webgpu.md`](docs/scheme/roadmap-wasi-webgpu.md)

### Docs — long-term plan + archive short-term M0–M5 (2026-08-11)

- Archive short-term thin L1 path: [`docs/scheme/archive/m0-m5-thin-l1.md`](docs/scheme/archive/m0-m5-thin-l1.md); freeze [`milestones.md`](docs/scheme/milestones.md) as history
- Add long-term plan: [`docs/scheme/long-term-plan.md`](docs/scheme/long-term-plan.md) (P0 wasi:webgpu · P1 WASI 0.3 · P2 Wasmtime tracking; stacks L0–L5)
- Add [`wasi-p3-surface.md`](docs/scheme/wasi-p3-surface.md), [`roadmap-wasi-webgpu.md`](docs/scheme/roadmap-wasi-webgpu.md), [`wasmtime-tracking.md`](docs/scheme/wasmtime-tracking.md)
- Add [`vcs-workflow.md`](docs/scheme/vcs-workflow.md): short-lived branches + PR; no long-lived parallel feature lines; open-source PR readiness
- Supersede [`rfc-wasi-worlds.md`](docs/scheme/rfc-wasi-worlds.md); revise [`non-goals.md`](docs/scheme/non-goals.md) NG-4/NG-5/NG-11 for long-term
- Update charter / scheme index / contribute / root README (ZH+EN) — **docs only**, no code

### Docs — M5 close: contributor shell + WASI worlds RFC (2026-08-11)

- Add [`docs/contribute.md`](docs/contribute.md): contributor flow; optional desktop JVM shell
- Add `scripts/build-native-host.ps1` → `desktop/jniLibs/`; JVM smoke `:runtime-jni:test` (`HostLoadLibraryTest`)
- Add [`docs/scheme/rfc-wasi-worlds.md`](docs/scheme/rfc-wasi-worlds.md): conditional WASI world roadmap (P0 webgpu subset; P2 needs new RFC)
- Mark M5 DoD complete in [`docs/scheme/milestones.md`](docs/scheme/milestones.md)

### Docs — M5 slice: API stability policy (2026-08-11)

- Add [`docs/scheme/api-stability.md`](docs/scheme/api-stability.md): `0.x-experimental` SemVer rules, surface tiers, Track A follow, 1.0 gate
- Wire `smoke-app` `versionName` to `wasmtime.android.version` in root `gradle.properties`

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
