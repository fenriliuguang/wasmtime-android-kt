# Agent playbook: `0.1.0` product gates

**English** | [中文](product-010.zh.md)

P0 `wasi:webgpu` is **closed**. P1 WASI 0.3 official-shape is **closed**. Do **not** re-cut W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, G-dev, or wasi:webgpu G1–G9 / F1–F9 / guest-pipeline / WG-6.

This queue lands the **L5 `0.1.0` product subset** ([`../scheme/rfc-l5-productization.md`](../scheme/rfc-l5-productization.md) §7–§8) plus the gfx present loop ([`../scheme/rfc-wasi-gfx-frame-loop.md`](../scheme/rfc-wasi-gfx-frame-loop.md)). Tracking needles: [`../scheme/product-010.md`](../scheme/product-010.md). One lane, one PR.

P2 Wasmtime pin stays a **named** queue ([`wasmtime-p2.md`](wasmtime-p2.md)). It is **not** this script’s `Next:`.

## Goal

A third party can depend on the **product subset** (webgpu most-of-pin + IO/network that webgpu apps need + **complete** gfx `on-frame` loop) at coordinate **`0.1.0`**. Complete loop = product `request-adapter` / `request-device` **and** Choreographer vsync into `on-frame`. Two pre-buffered frames (P010-GFXL) is **not** enough. **Also:** a **named physical-device** on-screen row, and root README **links an out-of-tree demo** (guest wasm → this Android runtime → present). Linking the demo is enough — do **not** vendor it here. Not wasi-testsuite (NG-4). Not CTS (NG-5). **P010-PUB** already landed publishing CI; do not press Central/Packages when secrets are missing.

## Select the cut

If the user named a lane (`P010-SPI`, `0.1.0`, `下一刀`), keep **one** family. Otherwise:

```powershell
.\scripts\product-010-remaining.ps1
```

No `pwsh`: `python3 ./scripts/product-010-remaining.py` (same flags: `--all`).

Do the printed **Next:** line only.

## Hard bans

- Do **not** re-cut P0 wasi:webgpu or P1 WASI 0.3 auto knives. One-shot WG-6 present stays a regression, not the product loop.
- Do **not** add `wasmtime-wasi` as a Cargo dependency unless that PR’s changelog records a size + Android thread review.
- Do **not** introduce `ai.tegmentum:wasmtime4j` or 4j native as the runtime.
- Do **not** JS-style `start: func(callback: func(ts: u64) -> bool)` for the frame loop.
- Do **not** publish Maven Central / GitHub Packages from a feature lane. **P010-PUB** owns `.github/workflows/publish*.yml`. No `0.0.x-preview`.
- Do **not** edit hub files on a feature lane: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`. Exceptions: this playbook / gate-amendment PR; **P010-PUB** (publish workflow); **P010-DEMO** (README Demo section only).
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** treat Latch / sync-compat as true CM async (NG-8).
- Do **not** read `native/src/cm.rs` without an offset. Grep the symbol (`ExperimentalHostCallbacks`, `wasi:sockets`, `wasi:http`, `get-gpu`), then Read ~80 lines.
- Never file GitHub issues on Wasmtime, WASI, wasi-webgpu, wasi-gfx, or any other upstream. No `gh issue create`.

Keep true `async` wraps (`func_wrap_concurrent` + yield) for WIT `async func`. `rustc` **1.97.1**.

## Lanes (auto)

Copy: L5 §7–§8 and the gfx RFC. Do not rediscover scope from RFCs by rewriting this table mid-cut.

| PR | Sentinel (needle in [`../scheme/product-010.md`](../scheme/product-010.md)) | DoD |
|----|----------------------------------------------------------------------------|-----|
| **P010-SPI** | `gap: p010 spi pending` | Move `ExperimentalHostCallbacks` **out of `runtime` public SPI** (sink into `:host-dawn`). Replace `WebGpuBackend.hostCallbacks()` leak with an internal attach. Remove `Store.setHostAdd` / `setRequestAdapter` / `setExperimentalHost` from product javadoc. Existing Dawn instrument still compiles. Changelog fragment. Remove the needle. |
| **P010-DISC** | `gap: p010 disc pending` | Dual-track: explicit `setWebGpuBackend` always wins. If today’s `discoverWebGpuBackend = false` stays, add a **new** factory (e.g. `Store.createWithDiscoveredBackend`). Zero factories → `request-adapter` `none`. Keep `:host-dawn` / `:android-webgpu` `consumer-rules.pro`. Device: existing Dawn smoke still runs with explicit attach. Remove the needle. |
| **P010-FIX** | `gap: p010 fix pending` | WebGPU **fixture** constructors (`get-gpu`, `get-device`, `get-gpu-error`, `get-device-lost-info`) leave the **product** linker. Instruments / `native/tests` may keep a test-only linker. Product path still chains `gpu.request-adapter`. Device: one existing Dawn product-path instrument (not a fixture ctor) still listed. Remove the needle. |
| **P010-CLIERR** | `gap: p010 cli-err pending` | Guest-visible `error-code` on **product** cli stdio / `run` paths (G-cli-error / G-err **as needed** — not the full enum dump). Device: extend a `WasiCli*` instrument with one err path. Remove the needle. |
| **P010-TCP** | `gap: p010 tcp pending` | **Outbound TCP**: guest `connect(ip-socket-address)` to a **non-loopback** IPv4; host **dials that address** (not ignore-port + echo pair). listen/UDP **out**. INTERNET already required. Device: new or extended `WasiSockets*` instrument. Changelog: sandbox (no listen by default). Remove the needle. |
| **P010-HBODY** | `gap: p010 http-body pending` | HTTP **body `stream<u8>`** on request/response (G-http-body types). In-process 200-only is not enough for this lane’s types. Device: extend `WasiHttp*` (or add) so a guest reads/writes a body stream. Remove the needle. |
| **P010-HOUT** | `gap: p010 http-out pending` | **Outbound** HTTP: `outgoing-handler` / `send` (or equivalent) to a real destination, not in-process 200-only. System trust. Device instrument. Remove the needle. |
| **P010-HCTOR** | `gap: p010 http-ctor pending` | Drop `[constructor]request` / `[constructor]response` from the **product** types surface (G-http-ctor). Host supplies `request` when calling `handle`. Test linker may keep constructors. Device: product-path handle still works. Remove the needle. |
| **P010-GFXP** | `gap: p010 gfx-pin pending` | Vendor **one dated** `wasi-gfx` WIT under `third_party/` (like `wasi:webgpu@0.3.0-rc.2`). Changelog names the tag. No host loop yet. Do **not** pick a second tag in a later lane without a changelog. Remove the needle. |
| **P010-GFXH** | `gap: p010 gfx-host pending` | Host `wasi-gfx:surface` + `on-frame` **CM stream** write on **GpuThread**. Guest **pulls**; no JS callback. Thread rules: [`../mapping/threading-android.md`](../mapping/threading-android.md). Native smoke that the stream yields. Remove the needle. |
| **P010-GFXL** | `gap: p010 gfx-loop pending` | **Skeleton (landed).** Product guest: async `run` loops `on-frame` → `get-current-texture` → submit → present (two **pre-buffered** frames). Device instrument. **Not** vsync-paced; GPU bootstrap may still use fixture `get-device`. Complete loop is **P010-GFXB** then **P010-GFXV**. Remove the needle. |
| **P010-GFXB** | `gap: p010 gfx-boot pending` | **Landed.** Frame-loop **guest** chains pin `get-gpu` → `gpu.request-adapter` → `gpu-adapter.request-device` (Dawn). Instrument uses **`Linker.create`**, not `createWithFixtureConstructors` / `get-device`. Still may pre-buffer frames. Device: extend `WasiGfxFrameLoopInstrumentedTest`. Changelog. Remove the needle. |
| **P010-GFXV** | `gap: p010 gfx-vsync pending` | Host writes `on-frame` from **Choreographer** vsync posted to **GpuThread** ([`../mapping/frame-loop-suggestion.md`](../mapping/frame-loop-suggestion.md) §2). Drop the beat if the previous event is unconsumed (no unbounded queue). Guest still pulls; no JS callback. Device: multi-frame on-screen **paced by vsync** (not two events at construct). `surfaceDestroyed` closes the stream so `run` unblocks. Remove the needle. |
| **P010-DEMO** | `gap: p010 demo pending` | **Last auto cut.** (1) Root `README.md` / `README.zh.md` **Demo** section links **one out-of-tree** repo whose pipeline is: package guest wasm → this Android runtime (`android-webgpu` or source composite) → present on a Surface. **Introducing the link is enough** — do not add a demo module, submodule, or vendor the app here. `:smoke-app` is instruments, not this demo. Not wasmtime4j / Track A cube. (2) [`claim-010.md`](../scheme/claim-010.md) names **one physical device** (ABI + Android version) that ran on-screen wasm present (GFXV instrument or the linked demo). Cloud has **no** device: do not fail the PR for missing `connectedAndroidTest`; the needle stays until the README link **and** the device row are written. Changelog. Remove the needle. |
| **P010-CLAIM** | `gap: p010 claim pending` | Release-notes-shaped **claim table**: most pinned `wasi:webgpu` `[method]` names instantiate; androidx holes listed ([`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md)); WASI product subset vs named-only. Still **not** CTS. Docs + changelog only. Remove the needle. |
| **P010-PUB** | `gap: p010 publish pending` | Publishing CI (Maven Central + GitHub Packages, same GAV). Version **`0.1.0`** (drop `-experimental`). Do not press publish if secrets are missing — still land the workflow + coordinates. Hub files allowed **this lane** for publish workflow. **Not last:** complete loop is GFXB + GFXV; last auto cut is **P010-DEMO**. Remove the needle. |

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| P2 Wasmtime pin / patch / major RFC | User named P2 / wasmtime | [`wasmtime-p2.md`](wasmtime-p2.md) |
| G-cmd / G-fs-full / listen / UDP | User named those IDs | Still not `0.1.0` gates |
| Enable `wasmtime-wasi` | User named wasmtime-wasi | Size + Android thread review |
| Full wasi-testsuite / CTS | User named those | NG-4 / NG-5 |
| This-repo 1.0.0 | User named 1.0 | Upstream gates in L5 §6 first |

## File whitelist (typical IO / SPI lane)

- `native/src/cm.rs` — this package’s `linker.instance` only (windowed)
- `native/tests/wasi_*.rs` or `native/tests/webgpu_*.rs` — this family
- `fixtures/wasi/*` or `fixtures/p3/*` — this fixture only
- `smoke-app/src/androidTest/java/…/*InstrumentedTest.kt` — **required** for CLIERR / TCP / HTTP / GFXL / GFXB / GFXV; SPI / DISC / FIX reuse an existing Dawn product-path class
- `runtime-api/` / `runtime-jni/` / `host-dawn/` — SPI / DISC only as the DoD names
- `docs/scheme/product-010.md` — **remove this lane’s needle**
- `docs/mapping/gap-wasi-p3-wit.md` — that leftover’s **one row** (Named → Smoke) when a G-* row lands
- `docs/mapping/threading-android.md` — TCP / HTTP / gfx thread policy
- `changelog/unreleased/<yyyy-mm-dd>-p010-<slug>.md`

GFX-PIN may add `third_party/wasi-gfx/…`. P010-PUB may add `.github/workflows/publish*.yml` and `gradle.properties` version.

Do not add files under `docs/archive/`. Do not re-cut webgpu G1–G9 fixtures.

## Narrow tests

```powershell
cd native
cargo check --locked --lib
```

Plus the **one** `cargo test --locked --test <this_slice>` the lane adds or extends.

Device (required where the table says Device):

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest -Pandroid.testInstrumentationRunnerArguments.class=<instrument>
```

Cloud has **no** device. Still **add** the instrument; do not fail the PR solely because this checkout could not run it. State that in the PR.

This playbook amendment (docs-only): `python3 ./scripts/product-010-remaining.py` must print **`Next: P010-GFXB`**.

## PR title

- This open: `docs: start 0.1.0 product playbook and remaining queue`
- SPI / DISC / FIX: `refactor(api): P010 …` or `feat(runtime): P010 …`
- WASI IO: `feat(wasi): P010 …`
- Gfx: `feat(gfx): P010 …`
- Claim / publish: `docs: P010 claim table` / `chore: P010 publish 0.1.0`
- This amendment: `docs: narrow 0.1.0 gates to complete gfx frame loop`
- Demo README link: `docs: P010 out-of-tree demo and device row`

Label: `documentation` for docs-only; `enhancement` for code.

## Copy source

[`../scheme/rfc-l5-productization.md`](../scheme/rfc-l5-productization.md) §7–§8, [`../scheme/rfc-wasi-gfx-frame-loop.md`](../scheme/rfc-wasi-gfx-frame-loop.md), leftover names on [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md). Do not invent extra `0.1.0` gates beyond **GFXB / GFXV / DEMO** (G-cmd, testsuite, `wasmtime-wasi`). Complete frame loop + out-of-tree demo link + named device row are in-scope; full desktop gfx stays DG-6. Do not vendor the demo into this repo.
