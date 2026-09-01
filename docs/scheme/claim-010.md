# `0.1.0` claim table (not CTS)

**English** | [中文](claim-010.zh.md)

Release-notes-shaped **product subset** for L5 [`rfc-l5-productization.md`](rfc-l5-productization.md) §7–§8. `0.1.0` playbook: [`../agent/product-010.md`](../agent/product-010.md) (empty). Living consume rewrite: [`../agent/native-dawn.md`](../agent/native-dawn.md). Counted 2026-08-27 against pin WIT + `native/src/cm.rs`.

**This is not a compliance claim.** No WebGPU CTS. No “full WASI 0.3”. No this-repo 1.0. See [`non-goals.md`](non-goals.md) NG-4 / NG-5. L5 subset landed at **`0.1.0`**; default consume is Dawn C at **`0.2.0`**. Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml) (do not press when secrets / arm64 `.so` are missing).

## 1. One-line claim

A third party can depend on this host for **most of pinned `wasi:webgpu@0.3.0-rc.2`** (guest `[method]` names consume on **Dawn C** / NativeGpu, not “instantiate via JNI”) plus the **WASI 0.3 IO/network subset webgpu apps need** and a **complete `wasi-gfx` present loop** (product adapter/device + vsync `on-frame`). **P010-GFXV** landed Choreographer vsync (1-slot; close on `surfaceDestroyed`). **P010-DEMO** links an out-of-tree wasm→runtime→present repo and names one physical-device on-screen row in §6. Record holes and named-only WASI leftovers are listed, not silent.

## 2. `wasi:webgpu` (pin `0.3.0-rc.2`)

Pin: [`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit). Shape rules: [`guest-shape.md`](guest-shape.md). Living NativeGpu holes: [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md). JNI leftover map: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md).

| Claim | Degree | Notes |
|-------|--------|-------|
| Pin resource `[method]` names (224 in WIT) | **Dawn C** (NativeGpu) | All 224 reconstructed `[method]resource.name` strings are registered in `native/src/cm.rs`. Default `GpuBackends.dawn()` consumes in-process (table-backed until `libwebgpu_dawn.so` binds slots). JNI leftover is `dawn-jni`. Not CTS. |
| `gpu.request-adapter` / `gpu-adapter.request-device` | **Dawn C** on `dawn()`; **`none` / err** if unwired | Product `Linker.create` exports pin `get-gpu`. Fixture `get-device` omitted. Unwired store → adapter **`none`**, not a trap. P010-GFXB: frame-loop guest uses this chain. |
| Compute submit / guest-drawn 3D / one-shot canvas | **Dawn C** (table) | WG-6 instruments stay as regressions; **ND-DEVICE** lists them on the native default. |
| Continuous on-screen loop | **Host vsync** | P010-GFXB: product `request-adapter` / `request-device`. P010-GFXV: Choreographer vsync into `on-frame` (drop unconsumed; `surfaceDestroyed` closes). Named device row is §6. [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md). |
| Out-of-tree demo + device-verified | **Smoke** | README **Demo** links [wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples) (pack guest wasm → this Android runtime → present). Linking is existence; not vendored. Named row in §6. |
| Dawn C / androidx missing ctor slots | **Record** (not Dawn C) | Same three: shader `compilation-hints`; canvas `color-space`; canvas `tone-mapping`. Native: [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md) §2. JNI leftover: androidx gap §2. |
| Fixture `get-*` constructors | **Not product** | `get-device` / `get-gpu-error` / `get-device-lost-info` stay on `Linker.createWithFixtureConstructors`. Pin `get-gpu` is product (WIT). |
| `experimental:webgpu-cm` flat `u32` names | **Not pin product** | Frozen dual-register; do not extend. |

**Dawn C** ≠ every field reaches `webgpu.h`. Record leftovers keep the guest value on `NativeGpuHost` (and on the JNI leftover record) when Dawn C / the AAR has no slot. Do not silently drop pin methods. The rotating cube is **demo** evidence, not this table’s consume degree. Device instruments on the native default are **ND-DEVICE**.

## 3. WASI 0.3 product subset vs named-only

Leftover WIT: [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md). Default host is the in-tree thin host (**not** `wasmtime-wasi`).

| Package | Product (`0.1.0`) | Named-only (not this gate) |
|---------|-------------------|----------------------------|
| CM stream / future, `wasi:clocks` instant, `wasi:random` | Landed functions | — |
| `wasi:cli` | stdio + `run`; guest-visible `illegal-byte-sequence` on NUL stdout/stderr; `error-code` includes `io` / `pipe` / `unknown` | Full enum dump (**G-err**); full `command` world (**G-cmd**) |
| `wasi:filesystem` | Preopen directory + `open-at` + read/write; `..` / absolute / NUL → `access` | `stat` / dir stream / append / dates (**G-fs-full**) |
| `wasi:sockets` | Outbound TCP `connect(ip-socket-address)` dials guest non-loopback IPv4 | listen, UDP, DNS (**G-sock-rest**) |
| `wasi:http` | Body `stream<u8>`; outbound `client#send` HTTP/1.1 GET; product linker omits `[constructor]request` / `[constructor]response` | `service` world, trailers, TLS crate / https |
| `wasi-gfx` | `surface@0.2.0` + `on-frame` stream + `surface-webgpu` present. **P010-GFXB** product adapter/device. **P010-GFXV** Choreographer vsync (1-slot; close on `surfaceDestroyed`). **P010-DEMO** README out-of-tree demo + this table’s device row | Full desktop gfx / multi-window (DG-6) |

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

| Device | ABI | Android | Path | Date |
|--------|-----|---------|------|------|
| Vivo V2458A (PD2415M) | arm64-v8a | 16 | `WasiGfxFrameLoopInstrumentedTest` (P010-GFXV vsync-paced present) | 2026-08-27 |

Cloud has **no** device. This is one named pass, not a device matrix. Not CTS. Out-of-tree demo: [wasmtime-android-kt-examples](https://github.com/fenriliuguang/wasmtime-android-kt-examples).
