# RFC: wasi-gfx frame loop (`0.1.0` gate)

**Status: Accepted (intent)** · 2026-08-26  
**English** | [中文](rfc-wasi-gfx-frame-loop.zh.md)

> Companion to [`rfc-l5-productization.md`](rfc-l5-productization.md) §7–§8.  
> Shape notes (not this RFC’s DoD): [`../mapping/frame-loop-suggestion.md`](../mapping/frame-loop-suggestion.md).  
> Does **not** reopen P0 wasi:webgpu (G1–G9 / WG-6). Does **not** make gfx a new **P0**. Auto lanes: [`product-010.md`](product-010.md) **P010-GFXP / GFXH / GFXL** (not `wasmtime-p2-remaining`).

## 1. Decision

Continuous on-screen present for **`0.1.0`** uses **`wasi-gfx`**, not a JS-style scheduler inside `wasi:webgpu`.

| Question | Decision |
|----------|----------|
| Is gfx P0? | **No.** P0 wasi:webgpu stays closed. |
| Is gfx a `0.1.0` product gate? | **Yes.** One-shot WG-6 present remains a regression, not the product story. |
| Guest clock | Guest **pulls** an `on-frame` **CM `stream`**. Host writes vsync on **GpuThread**. |
| Forbidden shape | `start: func(callback: func(ts: u64) -> bool)` — re-enters the guest from Choreographer and fights one `run_concurrent` driver per `Store`. |
| Pin | **`wasi-gfx:surface@0.2.0`** (tag `v0.2.0`, P010-GFXP). Vendored [`../../third_party/wasi-gfx/v0.2.0/wit/surface.wit`](../../third_party/wasi-gfx/v0.2.0/wit/surface.wit). Names may move; the pin does not. Host loop is P010-GFXH. |
| World | `run` stays **async** so reading `on-frame` can yield. |

## 2. Why a separate RFC

L5 names the gate; this page names the **guest/host shape** so later knives do not invent a callback package or reopen WebGPU leftover descriptors. Upstream [wasi-gfx](https://github.com/wasi-gfx/wasi-gfx) is still a proposal. This repo vendors tag **`v0.2.0`** (`surface` + `surface-webgpu` importing `wasi:webgpu@0.3.0-rc.2`).

## 3. Architecture (normative sketch)

```text
Guest  async run
       loop: read on-frame → get-current-texture → encode → submit → present
WIT    wasi-gfx:surface            stream<frame-event>
       wasi-gfx:surface-webgpu     swapchain glue
       wasi:webgpu@<pin>           GPU objects; no rAF
Host   one Store, one run_concurrent driver
       GpuThread: Dawn, GPUSurface, stream write, present
       UI thread: Surface lifecycle + vsync post; no Dawn
```

Thread rules stay [`../mapping/threading-android.md`](../mapping/threading-android.md).

## 4. Out of this RFC

- Multi-window / full wasi-gfx desktop (still DG-6 beyond a **minimal** present loop).
- CTS, rewriting Dawn, host-fixed `u32` as acceptance.
- Auto-queue on `wasmtime-p2-remaining.py`.
- A second WIT tag (changelog + replace `third_party/wasi-gfx/v0.2.0/` first).

## 5. Follow-up

1. Pin + vendor WIT under `third_party/` (one tag) — **P010-GFXP** (`v0.2.0`) landed.  
2. Host `surface` + `on-frame` stream on GpuThread — **P010-GFXH** landed.  
3. Product guest path skeleton (GFXL) — **landed** (`get-device` bootstrap still fixture; two pre-buffered frames).  
4. Device instrument: multi-frame present (WG-6 one-shot stays) — **P010-GFXL** landed.  
5. Product adapter/device bootstrap (no fixture `get-device`) — **P010-GFXB**.  
6. Choreographer vsync into `on-frame` (drop unconsumed beats; close stream on `surfaceDestroyed`) — **P010-GFXV**.  
7. Out-of-tree demo README link + named device row — **P010-DEMO** (not developed in this repo; linking is existence).  
8. Auto: [`product-010.md`](product-010.md) **P010-GFXP → GFXH → GFXL → GFXB → GFXV → DEMO**.
