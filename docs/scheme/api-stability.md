# API stability (experimental)

**English** | [中文](api-stability.zh.md)

1. Stay on **`0.x.y`** until upstream 1.0 gates in [`rfc.md`](rfc.md) §6 (break in MINOR). This-repo **1.0.0** is not a calendar target.
2. SemVer 2.0 shape: `MAJOR.MINOR.PATCH[-prerelease]`.
3. Current coordinate: **`0.1.1`** (`gradle.properties` → `wasmtime.android.version`). Subsequent bumps follow the `0.x` rules below. Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml). Do not press when secrets, arm64 wasmtime jniLibs, or arm64 `libwebgpu_dawn.so` are missing.
4. No CTS / compliant wasi:webgpu claim ([`non-goals.md`](non-goals.md) NG-5).
5. `ExperimentalHostCallbacks` is **not** `runtime-api` public SPI. Attach with `Store.setWebGpuBackend`.
6. Dual-track: `Store.setWebGpuBackend` is the stable contract; `Store.createWithDiscoveredBackend` is default-bundle convenience.
7. Product `Linker.create` omits fixture constructors (`get-device` / `get-gpu-error` / `get-device-lost-info`) and HTTP `[constructor]request` / `[constructor]response`. Pin `get-gpu` is product.
8. Guest pins: `wasi:webgpu@0.3.0-rc.2`, `wasi-gfx:surface@0.2.0`. Product `GpuBackends.dawn()` is NativeGpu. `dawn-jni` is leftover.

## `0.x` rules

Breaking public Kotlin/JNI/error semantics: at least `0.MINOR+1.0` **and** a changelog fragment. Compatible additions: MINOR or PATCH. `0.x` does **not** freeze API within a MINOR. `wasmtime` **47.x patch** → this-repo PATCH; **47→48** still needs a short RFC and may land as **`0.MINOR`**.

| Layer | Stability |
|-------|-----------|
| Public Kotlin (`Engine` / `Store` / …) | Breakable; bump MINOR |
| JNI `external` signatures | Same as public API |
| Leftover flat imports | Most unstable |
| Guest fixtures / instruments | Not library API |
