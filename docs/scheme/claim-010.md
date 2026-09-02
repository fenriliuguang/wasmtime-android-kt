# `0.1.0` claim table (not CTS)

**English** | [中文](claim-010.zh.md)

Release-notes-shaped **product subset**. Policy: [`rfc.md`](rfc.md). Remaining close-out: [`../agent/remaining.md`](../agent/remaining.md).

**This is not a compliance claim.** No WebGPU CTS. No “full WASI 0.3”. No this-repo 1.0. See [`non-goals.md`](non-goals.md). Coordinate **`0.1.0`** (not pressed). Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml).

## 1. One-line claim

A third party can depend on this host for **most of pinned `wasi:webgpu@0.3.0-rc.2`** (guest `[method]` names on **NativeGpu**) plus the **WASI 0.3 IO/network subset webgpu apps need** and a **wasi-gfx present loop** (product adapter/device + vsync `on-frame`). Record holes and named leftovers are listed, not silent.

Default consume is Dawn C / NativeGpu. Remaining pin `[method]`s call `webgpu.h` when `libwebgpu_dawn.so` is loaded; Cloud / missing `.so` stays table-backed ([`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md)). Cube is demo evidence only.

## 2. `wasi:webgpu` (pin `0.3.0-rc.2`)

| Claim | Degree | Notes |
|-------|--------|-------|
| Pin resource `[method]` names (224) | **Shape** + NativeGpu | All 224 names registered in `native/src/cm.rs`. Unwired store → `request-adapter` **`none`**. JNI leftover is `dawn-jni`. |
| Boot / cube hot path | **Dawn C** when `.so` loads | `request-adapter` / `request-device` / queue / buffer / WGSL / render pipeline / encoder / draw / submit / write-buffer / Android surface present. Guest options (features/limits/power) ignored on the C call. |
| Remaining pin methods | **Dawn** when `.so` loads | texture / sampler / compute / copies / map / query / bundle / error / indexed-indirect / viewport / `write-texture` / work-done / destroy. Cloud / missing `.so` stays **Table**. |
| Dawn C / AAR missing ctor slots | **Record** | shader `compilation-hints`; canvas `color-space`; canvas `tone-mapping` |
| Fixture `get-*` / `experimental:webgpu-cm` flats | **Not product** | Frozen; do not extend |

## 3. WASI 0.3 product subset vs named-only

Leftovers: [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md). Host is the in-tree thin host (**not** `wasmtime-wasi`).

| Package | Product (`0.1.0`) | Named-only |
|---------|-------------------|------------|
| CM stream / future, clocks, random | Landed functions | — |
| `wasi:cli` | stdio + `run`; NUL → `illegal-byte-sequence` | Full enum / `command` world |
| `wasi:filesystem` | Directory preopen + `open-at` + r/w; `..` → `access` | `stat` / dir stream / append |
| `wasi:sockets` | Outbound TCP IPv4 | listen, UDP, DNS |
| `wasi:http` | Body `stream<u8>`; outbound GET; no product request/response constructors | `service` world, trailers, TLS |
| `wasi-gfx` | `surface@0.2.0` constructor + `on-frame` + `configure` / `get-current-texture` / `present` | See remaining + non-urgent below |

**Remaining (auto):** surface size/resize; pin pointer/key streams.

**Non-urgent:** `context.unconfigure`; timestamped `frame-event`; Lost/Outdated `result`; multi-window.

Sandbox (documented promise, not a proof): FS = app-private; TCP = outbound + INTERNET, no listen by default; HTTP = system trust.

## 4. Public SPI

[`api-stability.md`](api-stability.md). Product: `Engine` / `Store` / `Linker` / `Component` / `Instance`. `ExperimentalHostCallbacks` is not public SPI. `Store.setWebGpuBackend` wins; `Store.createWithDiscoveredBackend` is discover convenience.

## 5. Explicitly not claimed

| Item | Why |
|------|-----|
| WebGPU CTS / “compliant wasi:webgpu” | NG-5 |
| Full wasi-testsuite P3 | NG-4 |
| This-repo **1.0.0** | Upstream gates in [`rfc.md`](rfc.md) §6 |
| `wasmtime-wasi` crate | Size + Android thread review |
| JS-style `start(callback)` | gfx loop is pull-stream |
| Maven press at this coordinate | Workflow exists; first press still needs secrets + arm64 `.so` |

## 6. Device-verified on-screen

Cloud has **no** device. Named, not a matrix. Cube is demo only.

| Instrument / path | Role |
|-------------------|------|
| `WasiGfxFrameLoopInstrumentedTest` | vsync-paced `on-frame` present |
| `WasiWebGpuDawnGuestComputeInstrumentedTest` | guest compute |
| `WasiWebGpuDawnGuestRenderInstrumentedTest` | guest 3D |
| `WasiWebGpuDawnGuestCanvasPresentInstrumentedTest` | guest-drawn canvas |
| `WasiWebGpuMethodCanvasContextPresentInstrumentedTest` | host-owned window |
| `WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest` | hitch recycle twin |

| Device | ABI | Android | Path | Date |
|--------|-----|---------|------|------|
| Vivo V2458A (PD2415M) | arm64-v8a | 16 | `WasiGfxFrameLoopInstrumentedTest` | 2026-08-27 |
| Vivo V2458A (PD2415M) | arm64-v8a | 16 | Out-of-tree cube ([examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples)) — demo | 2026-08-27 |
