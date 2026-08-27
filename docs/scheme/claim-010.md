# `0.1.0` claim table (not CTS)

**English** | [中文](claim-010.zh.md)

Release-notes-shaped **product subset** for L5 [`rfc-l5-productization.md`](rfc-l5-productization.md) §7–§8. Playbook: [`../agent/product-010.md`](../agent/product-010.md). Counted 2026-08-27 against pin WIT + `native/src/cm.rs`.

**This is not a compliance claim.** No WebGPU CTS. No “full WASI 0.3”. No this-repo 1.0. See [`non-goals.md`](non-goals.md) NG-4 / NG-5. Coordinate **`0.1.0`**. Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) (do not press when secrets / arm64 `.so` are missing).

## 1. One-line claim

A third party can depend on this host for **most of pinned `wasi:webgpu@0.3.0-rc.2`** (guest `[method]` names instantiate) plus the **WASI 0.3 IO/network subset webgpu apps need** and a **complete `wasi-gfx` present loop** (product adapter/device + vsync `on-frame`). Until **P010-GFXV**, vsync is not the cadence (two pre-buffered frames). Until **P010-DEMO**, there is no named device-verified row here and no README **Demo** link to an out-of-tree wasm→runtime→present app. Androidx holes and named-only WASI leftovers are listed, not silent.

## 2. `wasi:webgpu` (pin `0.3.0-rc.2`)

Pin: [`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit). Shape rules: [`guest-shape.md`](guest-shape.md). Living holes: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md).

| Claim | Degree | Notes |
|-------|--------|-------|
| Pin resource `[method]` names (224 in WIT) | **Instantiate** | All 224 reconstructed `[method]resource.name` strings are registered in `native/src/cm.rs`. A guest that imports them links. Not CTS. |
| `gpu.request-adapter` / `gpu-adapter.request-device` | **Dawn** on attached backend; **`none` / err** if unwired | Product `Linker.create` exports pin `get-gpu`. Fixture `get-device` omitted. Unwired store → adapter **`none`**, not a trap. P010-GFXB: frame-loop guest uses this chain. |
| Compute submit / guest-drawn 3D / one-shot canvas | **Dawn** | WG-6 instruments stay as regressions. |
| Continuous on-screen loop | **Incomplete** | P010-GFXB landed: product `request-adapter` / `request-device`. P010-GFXL: two pre-buffered frames. **P010-GFXV**: Choreographer vsync. [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md). |
| Out-of-tree demo + device-verified | **Incomplete** | **P010-DEMO**: root README **Demo** section links **one** repo that packs guest wasm → this Android runtime (`android-webgpu` or source composite) → present. Linking is existence; do not vendor. Plus one **named** row in §6 (ABI + Android version + GFXV instrument or that demo). |
| androidx.webgpu `1.0.0-alpha05` missing ctor slots | **Record** (not Dawn) | Only: shader `compilation-hints`; canvas `color-space`; canvas `tone-mapping`. See gap table §2. |
| Fixture `get-*` constructors | **Not product** | `get-device` / `get-gpu-error` / `get-device-lost-info` stay on `Linker.createWithFixtureConstructors`. Pin `get-gpu` is product (WIT). |
| `experimental:webgpu-cm` flat `u32` names | **Not pin product** | Frozen dual-register; do not extend. |

“Instantiate” ≠ every field reaches Dawn. Record leftovers keep the guest value in Kotlin and drop it at the AAR ctor.

## 3. WASI 0.3 product subset vs named-only

Leftover WIT: [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md). Default host is the in-tree thin host (**not** `wasmtime-wasi`).

| Package | Product (`0.1.0`) | Named-only (not this gate) |
|---------|-------------------|----------------------------|
| CM stream / future, `wasi:clocks` instant, `wasi:random` | Landed functions | — |
| `wasi:cli` | stdio + `run`; guest-visible `illegal-byte-sequence` on NUL stdout/stderr; `error-code` includes `io` / `pipe` / `unknown` | Full enum dump (**G-err**); full `command` world (**G-cmd**) |
| `wasi:filesystem` | Preopen directory + `open-at` + read/write; `..` / absolute / NUL → `access` | `stat` / dir stream / append / dates (**G-fs-full**) |
| `wasi:sockets` | Outbound TCP `connect(ip-socket-address)` dials guest non-loopback IPv4 | listen, UDP, DNS (**G-sock-rest**) |
| `wasi:http` | Body `stream<u8>`; outbound `client#send` HTTP/1.1 GET; product linker omits `[constructor]request` / `[constructor]response` | `service` world, trailers, TLS crate / https |
| `wasi-gfx` | `surface@0.2.0` + `on-frame` stream + `surface-webgpu` present (GFXL: two pre-buffered frames). **Complete loop pending:** product adapter/device (**P010-GFXB**) + vsync (**P010-GFXV**). Last auto cut **P010-DEMO** (README out-of-tree demo + this table’s device row) | Full desktop gfx / multi-window (DG-6) |

Sandbox (documented promise, not a proof): FS = app-private; TCP = outbound + INTERNET, no listen by default; HTTP = system trust, no bundled CA this cut.

## 4. Public SPI (already landed)

[`api-stability.md`](api-stability.md). Product: `Engine` / `Store` / `Linker` / `Component` / `Instance`. `ExperimentalHostCallbacks` is not `runtime-api` public SPI. Dual-track: `Store.setWebGpuBackend` wins; `Store.createWithDiscoveredBackend` is discover convenience.

## 5. Explicitly not claimed

| Item | Why |
|------|-----|
| WebGPU CTS / “compliant wasi:webgpu” | NG-5 |
| Full wasi-testsuite P3 | NG-4 |
| This-repo **1.0.0** / WASI 1.0 distro | Upstream gates in L5 §6 first |
| `wasmtime-wasi` crate | Size + Android thread review, named-only |
| JS-style `start(callback)` frame scheduler | gfx RFC |
| Maven Central / GitHub Packages at this coordinate | **P010-PUB landed.** Workflow + `0.1.0` GAV. First press still needs Portal token, in-memory GPG, and arm64 `libwasmtime_android_kt.so` |

## 6. Device-verified on-screen (P010-DEMO)

Cloud has **no** device. Until **P010-DEMO**, this table has **no** named physical-device row. That lane writes **one** row here (ABI + Android version + which on-screen path: GFXV instrument or the README-linked demo). Instrument history (V2458A arm64 Android 16, …) is not the gate by itself.
