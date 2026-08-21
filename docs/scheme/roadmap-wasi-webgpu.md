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
| WG-5 | Citable threading + mapping notes in this repo only; **never** file upstream GitHub issues |
| WG-6 | (Mid) render or compute slice; present via `gpu-canvas-context` after marshalling — not wasi-gfx |

## 3. Slice order

Live status: GitHub Project. Definitions: [`guest-shape.md`](guest-shape.md) S-series.

Do **not** open new host-fixed `u32` feature PRs.

Guest compute/3D marshalling (bind groups, vertex layouts, depth, texture extras) is closed: [`../agent/webgpu-guest-pipeline.md`](../agent/webgpu-guest-pipeline.md). Leftover optional descriptor fields + Dawn consume: [`../agent/webgpu-guest-semantics.md`](../agent/webgpu-guest-semantics.md).

**Present:** product path is proposal `gpu-canvas-context` (no `present` in wasi:webgpu). wasi-gfx remains a deferred RFC.

## 4. Upstream

**Do not** create, reopen, or request GitHub Issues (or GitHub Discussions used as an issue tracker) on wasi-webgpu, Wasmtime, or any other upstream. No `gh issue create`. Android-host facts stay in `changelog/unreleased/` and [`../mapping/threading-android.md`](../mapping/threading-android.md).

[wasi-webgpu#81](https://github.com/WebAssembly/wasi-webgpu/issues/81) was filed in error from this host (2026-08-21) and is retracted. Close it on GitHub if still open; do not cite it as protocol.

## 5. Out of scope on this roadmap

Compliance / full CTS; **rewriting** Dawn (NG-7); wasi-gfx as P0; fake async.
