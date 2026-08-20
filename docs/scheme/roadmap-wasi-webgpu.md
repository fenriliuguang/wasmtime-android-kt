# Roadmap: `wasi:webgpu` (P0)

**English** | [中文](roadmap-wasi-webgpu.zh.md)

Proposal: [WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) (Phase 2 at time of writing).  
Pin: `wasi:webgpu@0.3.0-rc.2`.  
Shape: [`guest-shape.md`](guest-shape.md). Plan: [`long-term-plan.md`](long-term-plan.md).

## 1. Why P0

Standard / proposal `wasi:webgpu` is full of WIT `async func`. This repo’s L1 is the Android path that can register those with **true** CM async. Windowing is out of scope for the proposal (`wasi-gfx` is NG-9 here).

## 2. Goals

| ID | Goal |
|----|------|
| WG-1 | Pin WIT (tag / commit) and keep guests aligned |
| WG-2 | Register critical `async func`s with true CM async (no Latch) |
| WG-3 | Own linker / resource / canonical marshalling; GPU backend is pluggable — **this repo owns the SPI**; default Dawn bundle ([`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)) |
| WG-4 | Device instruments: `gpu.request-adapter` / `gpu-adapter.request-device` in WIT shape |
| WG-5 | Citable threading + mapping notes; file upstream when Android-specific |
| WG-6 | (Mid) render or compute slice; present via `gpu-canvas-context` after marshalling — not wasi-gfx |

## 3. Slice order

Live status: GitHub Project. Definitions: [`guest-shape.md`](guest-shape.md) S-series.

Do **not** open new host-fixed `u32` feature PRs.

After default semantic-L2 remaining is 0: [`../agent/webgpu-midterm.md`](../agent/webgpu-midterm.md) (WG-6 canvas, S1–S3 guest fields, named `record-*`, citable Dawn / WG-5).

**Present:** product path is proposal `gpu-canvas-context` (no `present` in wasi:webgpu). wasi-gfx remains a deferred RFC.

## 4. Upstream

When an S-cluster lands with new Android-host information, open an issue on wasi-webgpu and/or Wasmtime and link it here.

| Date | Upstream | Note |
|------|----------|------|
| — | — | None yet under the 2026-08-17 protocol |

## 5. Out of scope on this roadmap

Compliance / full CTS; **rewriting** Dawn (NG-7); wasi-gfx as P0; fake async.
