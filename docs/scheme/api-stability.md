# API stability (experimental)

**English** | [中文](api-stability.zh.md)

1. Before **1.0.0** this repo is **experimental**.  
2. SemVer 2.0 shape: `MAJOR.MINOR.PATCH[-prerelease]`.  
3. Current: `0.1.0-experimental` (`gradle.properties` → `wasmtime.android.version`).  
4. No production / compliant wasi:webgpu claim ([`non-goals.md`](non-goals.md) NG-5).

## `0.x` rules

Breaking public Kotlin/JNI/error semantics: at least `0.MINOR+1.0` **and** a changelog fragment. Compatible additions: MINOR or PATCH. `0.x` does **not** freeze API within a MINOR.

| Layer | Stability |
|-------|-----------|
| Public Kotlin (`Engine` / `Store` / …) | Breakable; bump MINOR |
| JNI `external` signatures | Same as public API |
| `ExperimentalWebGpuBridge` / leftover flat imports | Most unstable |
| Guest fixtures / instruments | Not library API |

Guest product pin: `wasi:webgpu@0.3.0-rc.2`. Public GPU SPI (when landed) lives in `runtime-api`. Until the module-split PR, unpublished host Maven coordinates are listed in [`../blocked-gpu-host.md`](../blocked-gpu-host.md) and must be named in the changelog when bumped.
