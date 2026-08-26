# P0 close-out: `wasi:webgpu` (archived)

**English** | [中文](p0-wasi-webgpu.zh.md)

> **Closed 2026-08-22** on `main` (`0ed028b`, PR #253). Do not open another wasi:webgpu implementation queue. Living gap (WIT ↔ Kotlin ↔ androidx): [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md). Current work: [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md). P1 close-out: [`p1-wasi-p3.md`](p1-wasi-p3.md). Historical playbooks in this folder: `webgpu-guest-pipeline.md`, `webgpu-guest-semantics.md`, `webgpu-guest-dawn.md`.

Pin: `wasi:webgpu@0.3.0-rc.2` at [`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit). Shape rules stay in [`../scheme/guest-shape.md`](../scheme/guest-shape.md).

## Status

| Goal | Result |
|------|--------|
| WG-1 pin WIT | Vendored rc.2 |
| WG-2 true CM async | `func_wrap_concurrent` + yield; unwired `request-adapter` → guest `none` |
| WG-3 pluggable GPU SPI | `:host-dawn` + `:android-webgpu`; core AAR has no Dawn |
| WG-4 device instruments | smoke-app + WG-6 guest compute / render / canvas present |
| WG-5 citable notes | `changelog/unreleased/` + mapping; never file upstream GitHub issues |
| WG-6 guest-drawn slice | PRs #250–#252 |
| S1–S5 + S6+ hang | `[method]` names cover the pin (~225 resource methods) |
| F1–F9 leftover JNI | Closed |
| G1–G9 Dawn consume | Closed (#241–#249, #253) |

Not claimed: CTS / compliant product (NG-5). Not P0: wasi-gfx (NG-9).

## Timeline

| When | What |
|------|------|
| 2026-08-11 | Thin L1: load Wasmtime, true async gate, Dawn on-screen smoke |
| 2026-08-13–16 | W1–W3 hang names (then freeze host-fixed `u32`); S1–S5 WIT shape |
| 2026-08-16 | Canonical-shape RFC; NG-12 |
| 2026-08-17 | Dawn default bundle; Host Kotlin vendored; English IA |
| 2026-08-18–20 | S6+ remaining methods; L2 described JNI (lift-only lane emptied) |
| 2026-08-21 | F1–F9; G1–G9 |
| 2026-08-22 | WG-6 real compute / 3D / guest-drawn present; stage-only storage limits; this close-out |

## Problems worth keeping

- **Host-fixed `u32` is not acceptance** for new slices (NG-12). Early W3 hung names with discarded guest args; S-series replaced that.
- **Do not file upstream GitHub issues.** [wasi-webgpu#81](https://github.com/WebAssembly/wasi-webgpu/issues/81) was filed in error (2026-08-21) and retracted.
- **ART / OEM:** `opt-level=0` Android cross made `stream.write` SIGSEGV the instrument process — default `opt-level=2`. Android 16 / some OEMs treat `startActivity` as background; instruments unblank + `am start -W`. Vulkan `backendType` required so CM `createSurface` does not hit `WINDOW_IN_USE`.
- **androidx holes** stay on the Kotlin record until the AAR grows slots — see the gap page. Do not re-cut G1–G9 to “wait for androidx”.
- **JDT LS** copied `.kt` into `**/bin/` if Gradle import was on.

## What remains (not a P0 queue)

androidx `1.0.0-alpha05` still has no `compilationHints` ctor and no canvas `color-space` / `tone-mapping` surface slots. Test-only `get-*` constructors and frozen experimental flat names stay. Next phase is P2 (upstream Wasmtime tracking), not more WebGPU lanes.
