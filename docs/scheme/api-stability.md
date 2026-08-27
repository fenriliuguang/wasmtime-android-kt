# API stability (experimental)

**English** | [中文](api-stability.zh.md)

1. Stay on **`0.x.y`** until upstream 1.0 gates in [`rfc-l5-productization.md`](rfc-l5-productization.md) §6 (three.js-style: **break in MINOR**). This-repo **1.0.0** is not a calendar target.  
2. SemVer 2.0 shape: `MAJOR.MINOR.PATCH[-prerelease]`.  
3. Current local coordinate: `0.1.0-experimental` (`gradle.properties` → `wasmtime.android.version`). **No Central / Packages until `0.1.0` product gates.**  
4. No CTS / compliant wasi:webgpu claim ([`non-goals.md`](non-goals.md) NG-5).  
5. **P010-SPI landed:** `ExperimentalHostCallbacks` is **not** `runtime-api` public SPI (lives in `:runtime-jni` `internal`). Attach with `Store.setWebGpuBackend`; no `WebGpuBackend.hostCallbacks()`.
6. **P010-DISC landed:** dual-track attach — `Store.setWebGpuBackend` is the stable contract; `Store.createWithDiscoveredBackend` is default-bundle ServiceLoader convenience (`Store.create` still defaults to no discover).
7. **P010-FIX landed:** product `Linker.create` omits fixture constructors `get-gpu` / `get-device` / `get-gpu-error` / `get-device-lost-info`. Instruments use `Linker.createWithFixtureConstructors`.
8. **P010-CLIERR landed:** product cli `error-code` includes `io` / `illegal-byte-sequence` / `pipe`; stdout/stderr NUL write is guest-visible `illegal-byte-sequence`.
9. **P010-TCP landed:** product `tcp-socket.connect` dials guest non-loopback IPv4; no listen / UDP by default.
10. **P010-HBODY landed:** product http types expose body `stream<u8>` (`consume-body` / `response.new`); still in-process, not wire.
11. **P010-HOUT landed:** product `wasi:http/client#send` does HTTP/1.1 GET on the wire; https / extra TLS crate not this cut.
12. **P010-HCTOR landed:** product `Linker.create` omits `[constructor]request` / `[constructor]response`. Host supplies `request` when calling `handle`. Test linker keeps the constructors.
13. **P010-GFXP landed:** guest gfx pin is `wasi-gfx:surface@0.2.0` (tag `v0.2.0` under `third_party/wasi-gfx/`). Host `on-frame` is P010-GFXH.
14. **P010-GFXH landed:** product `wasi-gfx:surface` constructor + `on-frame` CM stream. Vsync payload is produced on a helper thread named `GpuThread`. Guest pulls; no JS callback. Present loop is P010-GFXL.
15. **P010-GFXL landed:** product guest loops `on-frame` → `get-current-texture` → submit → `context.present` (two frames). `surface-webgpu` is hosted. GPU bootstrap still uses fixture `get-device`. WG-6 one-shot stays.
16. **P010-CLAIM landed:** release-notes claim table [`claim-010.md`](claim-010.md) — all 224 pin `[method]` names instantiate; androidx holes listed; WASI subset vs named-only. **Not** CTS.

## `0.x` rules

Breaking public Kotlin/JNI/error semantics: at least `0.MINOR+1.0` **and** a changelog fragment. Compatible additions: MINOR or PATCH. `0.x` does **not** freeze API within a MINOR. `wasmtime` **47.x patch** → this-repo PATCH; **47→48** still needs a short RFC and may land as **`0.MINOR`**.

| Layer | Stability |
|-------|-----------|
| Public Kotlin (`Engine` / `Store` / …) | Breakable; bump MINOR |
| JNI `external` signatures | Same as public API |
| `ExperimentalWebGpuBridge` / leftover flat imports | Most unstable |
| Guest fixtures / instruments | Not library API |

Guest product pin: `wasi:webgpu@0.3.0-rc.2`. Gfx pin: `wasi-gfx:surface@0.2.0`. Public GPU SPI lives in `runtime-api` (`WebGpuBackend`). Unpublished host Maven coordinates are listed in [`../blocked-gpu-host.md`](../blocked-gpu-host.md) (`:host-dawn` only) and must be named in the changelog when bumped.
