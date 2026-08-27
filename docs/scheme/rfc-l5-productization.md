# RFC: L5 productization (0.x forever until upstream 1.0)

**Status: Accepted** · 2026-08-26  
**English** | [中文](rfc-l5-productization.zh.md)

> Amends [`long-term-plan.md`](long-term-plan.md), [`charter.md`](charter.md), [`non-goals.md`](non-goals.md), [`api-stability.md`](api-stability.md), [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md), [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md).  
> Does **not** reopen P0 wasi:webgpu or P1 WASI auto knives. Does **not** bump `wasmtime`.  
> Frame loop: separate RFC [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md).

## 1. Decision

This repo is an **Android-first app runtime** (product class B), not a WASI distro and not “citation-only.” The Wasm ecosystem (especially this repo’s first-class world) still ships as **proposals**. Versioning follows a **three.js-style perpetual `0.x.y`**: break in MINOR; do **not** ship **1.0.0** until the gates in §6.

| Question | Decision |
|----------|----------|
| Product class | **B** — App-consumable Android Component runtime. First class: **proposal-among-proposals** (`wasi:webgpu`, then `wasi-gfx`). |
| `0.x` vs `1.0.0` | Stay on **`0.x.y`** until `wasi:webgpu` and `wasi-gfx` are ratified WASI, WASI publishes **1.0**, and `androidx.webgpu` ships a **non-alpha** release. This repo’s 1.0 *implementation* list is a later discussion. |
| Engine | Official **`wasmtime` only**. Current generation **47.x** (patch OK). True **CM async only** — no WASI 0.2 pollable path. |
| WASI claim | **Product subset**, not wasi-testsuite / “full WASI 0.3”. `0.x` grows by usage. **`0.1.0` gate** = first-class webgpu + the IO/network subset that webgpu apps need (see §7). |
| WebGPU claim | `0.1.0` must claim **most of the pinned `wasi:webgpu` WIT**. Missing slots because the Dawn/androidx **backend has no field** are documented limits, not silent drops. **Not** CTS / compliant product (NG-5). |
| Publish | **Maven Central + GitHub Packages**, same coordinates. **No publish until `0.1.0` gates.** No `0.0.x-preview` Central. AAR ships `libwasmtime_android_kt.so`; apps may still rebuild natives. Dawn `.so` stays a **transitive** `androidx.webgpu` dependency. |
| Coordinates | **Three** artifacts; `0.x` default is **`android-webgpu`**. |
| GPU attach | **Dual track:** explicit `setWebGpuBackend` is the stable contract; ServiceLoader discover is the default-bundle convenience. |
| Public SPI | Before `0.1.0`, **move `ExperimentalHostCallbacks` out of `runtime` public SPI**. |
| Fixtures | **Tests only** — not in the product linker / not in published AARs. |
| Frame loop | **`0.1.0` requires a wasi-gfx-based loop** — [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md). Not a P0 re-cut. |

## 2. Why

P0/P1 closed a **Smoke** host that outsiders can cite. Product class B needs coordinates, a claimed subset, and a 0.1 gate — without pretending the proposal worlds are WASI 1.0. Perpetual `0.x` keeps MINOR breaking honest until upstream ratification.

## 3. Claims (0.x)

**May say**

- Official `wasmtime` embed (47.x; patch per [`wasmtime-tracking.md`](wasmtime-tracking.md)).
- True CM async is the **only** async.
- `wasi:webgpu` is first class on the **pinned proposal WIT**; most methods instantiate; androidx/Dawn holes are listed.
- WASI 0.3 **product subset** (table in the release notes), not WASI 1.0.

**Must not say**

- Full wasi-testsuite / “complete WASI 0.3”.
- Compliant WebGPU / CTS.
- Production SLA / frozen 1.0 API.
- Maven coordinates before `0.1.0` gates.

## 4. Publish and groupId

| Item | Value |
|------|--------|
| groupId | **`io.github.fenriliuguang.wasmtime.android`** (already in `gradle.properties`) |
| Versions | **`0.MINOR.PATCH` releases**, not Maven `*-SNAPSHOT` timestamps. GitHub Packages mirrors the same GAV. |
| First Central | **`0.1.0` after §7 + gfx RFC gates.** P010-PUB (2026-08-27): workflow [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml); do not press if secrets or arm64 `.so` are missing. No `0.0.x-preview`. |
| Natives in AAR | `android/jniLibs/<abi>/libwasmtime_android_kt.so` (arm64-v8a required; x86_64 for emulator in 0.x). |
| DIY natives | `scripts/build-native-android.ps1` remains documented. |
| Dawn | `:host-dawn` `api(androidx.webgpu:webgpu)` — do not git Dawn `.so`. |
| Tags | `v0.x.y` matches the coordinate version. |

### 4.1 Artifacts

| artifactId | Module | 0.x default? |
|------------|--------|----------------|
| `runtime` | `:android` (JNI + Wasmtime `.so` + SPI, **no** Dawn) | No (BYO / no GPU) |
| `host-dawn` | `:host-dawn` | No |
| `android-webgpu` | `:android-webgpu` (`api` of both) | **Yes — README recommends only this** |

Do **not** advertise `:runtime-api` as a fourth **consumer** coordinate (`api` of `runtime`). `runtime-api` and `runtime-jni` are published only so Maven can resolve `runtime`'s POM. Never publish `:smoke-app`.

`0.x` MINOR may break; PATCH is fix / `wasmtime` 47.x patch. Changelog every time.

## 5. GPU attach (dual track)

1. **Stable API:** `store.setWebGpuBackend(backend)` / `GpuBackends.dawn()`. BYO, tests, multi-backend **only** this path. Explicit always wins over discover.
2. **Default-bundle convenience:** apps on `android-webgpu` may `Store.create` with **discover = true** (prefer a **new** factory such as `Store.createWithDiscoveredBackend` if changing today’s `discoverWebGpuBackend = false` default is too sharp). Zero factories → `request-adapter` **`none`**. Several factories → prefer `id == "dawn"`.
3. **R8:** `consumer-rules.pro` on `:host-dawn` / `:android-webgpu` is part of the **published** contract (keep `WebGpuBackendFactory` + Dawn factory). README: minify must consume consumer Proguard.
4. Still forbidden: downloading `.so` at runtime; Play Feature hot-plug.

## 6. This-repo `1.0.0` (upstream gates only)

Do not start a 1.0 implementation list in this RFC. **Pre-conditions** (all required):

1. `wasi:webgpu` is ratified WASI (not a proposal pin).
2. `wasi-gfx` is ratified WASI.
3. WASI publishes **1.0**.
4. `androidx.webgpu` ships a **stable** (non-alpha) artifact.

Until then: perpetual `0.x.y`.

## 7. `0.1.0` product subset (not testsuite)

`0.x` after 0.1 may grow from community feedback. **`0.1.0` will not ship** until:

Living claim table (not CTS): [`claim-010.md`](claim-010.md).

| Area | `0.1.0` must |
|------|----------------|
| CM async / clocks / random / stream | Keep landed functions |
| `wasi:webgpu` | Most of the pin’s `[method]` names instantiate; Dawn path for compute / 3D / present; androidx holes **documented** |
| cli | stdio + `run` on the product path; guest-visible errors on those paths (G-err / G-cli-error as needed) |
| filesystem | Sandbox directory + `open-at` + read/write (today’s Smoke may stand; full `stat`/dir stream **not** required) |
| sockets | **Outbound TCP** for real app networking (not loopback-only). listen/UDP **not** required |
| http | **Body `stream<u8>`** and outbound send / outgoing-handler (or equivalent). In-process 200-only is **not** enough. Drop product `[constructor]request`/`response` (G-http-ctor) |
| gfx | Continuous on-screen loop: product `gpu.request-adapter` / `gpu-adapter.request-device` (**P010-GFXB**) **and** Choreographer vsync into `on-frame` (**P010-GFXV**) per [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md). Two pre-buffered frames (P010-GFXL) is **not** the gate. |
| demo / device | Root README **Demo** section links **one out-of-tree** repo whose pipeline is pack guest wasm → this Android runtime → present on a Surface (**P010-DEMO**). Linking counts as existence; do **not** vendor the demo here. Plus one **named physical-device** on-screen row in [`claim-010.md`](claim-010.md). |

**Not** `0.1.0` gates: full `wasi:cli/command` world (G-cmd), G-fs-full, wasi-testsuite, enabling `wasmtime-wasi` (still needs size + Android thread review).

P1 leftover table ([`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md)) stays **not** `wasmtime-p2-remaining` `Next:`. Rows that this §7 names are **auto** on [`product-010.md`](product-010.md), not a P1 re-queue.

## 8. Public API (before `0.1.0`)

**Product (javadoc; MINOR if removed):** `Engine`, `Store`, `Linker`, `Component`, `Instance`; `WasmtimeException` tree; `WebGpuBackend` / `WebGpuBackendFactory` / `WebGpuBackendKind`; `GpuBackends.dawn()`; `NativeLoader` / `NativeLibraryNames` / native version; thread contract as documented API.

**Remove from `runtime` public SPI before `0.1.0`:** `ExperimentalHostCallbacks` (sink into `:host-dawn`); `HostU32*` ; `Store.setHostAdd` / `setRequestAdapter` / `setExperimentalHost`; `NativeBridge`; `…experimental…` `WasiWebGpuHost`. Today `WebGpuBackend.hostCallbacks()` leaks the experimental table — replace with an internal attach.

**Never product:** fixtures, instruments, test constructors.

`0.x` may still change architecture (MINOR + changelog).

## 9. Support matrix (record toward 1.0)

Each `0.x` release notes: ABI tested (`arm64-v8a` official; `x86_64` emulator best-effort), minSdk **24**, device rows (keep history; V2458A arm64 Android 16 is the first), Cloud has **no** device, NDK/Rust pins. No `armeabi-v7a` / `x86` without a dedicated RFC.

## 10. Runtime model (default; 0.x may break)

- One `Store`, one `run_concurrent` driver.
- Dawn / `ANativeWindow` on **GpuThread** only.
- No heavy compile/instantiate on the ART main thread.
- Unwired GPU → guest `request-adapter` **`none`** (not a missing import, not a trap). Linker **always** defines `wasi:webgpu`.
- Thin JNI — do not clone a full Java Wasmtime API.
- Default WASI host is the in-tree thin host (not `wasmtime-wasi`).

## 11. Engine upgrades

`0.x` follows **47.x patches** (security GHSA → this repo PATCH). 47 → 48 still needs the tracking **short RFC**; because `0.x` allows breaking, that jump may land as **`0.MINOR`**, not this-repo 1.0. Release notes always name `wasmtime=`.

## 12. Size, license, sandbox

- Release notes: per-ABI bytes + sha256 (`build-info.json`). `runtime` has no Dawn `.so`. Default APK size = Wasmtime so + androidx Dawn so (order-of-magnitude, not a fake KB cap). Keep strip.
- AAR `NOTICE`: Apache-2.0 (this repo), Wasmtime LLVM-exception, vendored host Kotlin MIT, androidx.webgpu/Dawn. No experimental mvp coordinates in the POM.
- Default sandbox (documented promise, not a formal proof): FS = app-private only (`..`/absolute/NUL → `access`); TCP = outbound with INTERNET from `0.1.0` (no listen by default); HTTP outbound uses the product handler + Android system trust; GPU none vs Dawn as above.

## 13. Follow-up

Playbook / remaining: [`../agent/product-010.md`](../agent/product-010.md) (`python3 ./scripts/product-010-remaining.py`). Code lanes (SPI, dual-track factory, fixtures, IO, gfx, claim, publish) are that queue — not this RFC PR.

## 14. Non-goals this RFC does not lift

- CTS / compliant wasi:webgpu (NG-5).
- wasi-testsuite as KPI (NG-4).
- wasmtime4j (NG-2); rewriting Dawn (NG-7); host-fixed `u32` as new-slice DoD (NG-12).
- Promoting gfx to a **P0** wasi:webgpu re-queue (NG-9). Gfx is a **`0.1.0` product gate** via its own RFC.
