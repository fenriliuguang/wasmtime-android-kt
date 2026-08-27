# API stability (experimental)

**English** | [中文](api-stability.zh.md)

1. Stay on **`0.x.y`** until upstream 1.0 gates in [`rfc-l5-productization.md`](rfc-l5-productization.md) §6 (three.js-style: **break in MINOR**). This-repo **1.0.0** is not a calendar target.  
2. SemVer 2.0 shape: `MAJOR.MINOR.PATCH[-prerelease]`.  
3. Current local coordinate: `0.1.0-experimental` (`gradle.properties` → `wasmtime.android.version`). **No Central / Packages until `0.1.0` product gates.**  
4. No CTS / compliant wasi:webgpu claim ([`non-goals.md`](non-goals.md) NG-5).  
5. **P010-SPI landed:** `ExperimentalHostCallbacks` is **not** `runtime-api` public SPI (lives in `:runtime-jni` `internal`). Attach with `Store.setWebGpuBackend`; no `WebGpuBackend.hostCallbacks()`.
6. **P010-DISC landed:** dual-track attach — `Store.setWebGpuBackend` is the stable contract; `Store.createWithDiscoveredBackend` is default-bundle ServiceLoader convenience (`Store.create` still defaults to no discover).

## `0.x` rules

Breaking public Kotlin/JNI/error semantics: at least `0.MINOR+1.0` **and** a changelog fragment. Compatible additions: MINOR or PATCH. `0.x` does **not** freeze API within a MINOR. `wasmtime` **47.x patch** → this-repo PATCH; **47→48** still needs a short RFC and may land as **`0.MINOR`**.

| Layer | Stability |
|-------|-----------|
| Public Kotlin (`Engine` / `Store` / …) | Breakable; bump MINOR |
| JNI `external` signatures | Same as public API |
| `ExperimentalWebGpuBridge` / leftover flat imports | Most unstable |
| Guest fixtures / instruments | Not library API |

Guest product pin: `wasi:webgpu@0.3.0-rc.2`. Public GPU SPI lives in `runtime-api` (`WebGpuBackend`). Unpublished host Maven coordinates are listed in [`../blocked-gpu-host.md`](../blocked-gpu-host.md) (`:host-dawn` only) and must be named in the changelog when bumped.
