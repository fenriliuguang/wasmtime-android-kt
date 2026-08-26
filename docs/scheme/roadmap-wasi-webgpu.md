# Roadmap: `wasi:webgpu` (P0, closed)

**English** | [中文](roadmap-wasi-webgpu.zh.md)

Proposal: [WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu). Pin: `wasi:webgpu@0.3.0-rc.2`. Shape: [`guest-shape.md`](guest-shape.md). **Closed 2026-08-22** — [`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md). Remaining holes: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md). Current work is P2: [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md).

## Goals (done)

| ID | Goal |
|----|------|
| WG-1 | Pin WIT — vendored rc.2 |
| WG-2 | True CM async — `func_wrap_concurrent` |
| WG-3 | SPI + default Dawn bundle |
| WG-4 | Device instruments including WG-6 guest-drawn slices |
| WG-5 | Local notes only; never upstream GitHub issues |
| WG-6 | Guest-drawn compute / 3D / `gpu-canvas-context` present |

Do **not** open new host-fixed `u32` feature PRs. wasi-gfx is not P0 (NG-9). No CTS claim (NG-5).

## Upstream

**Do not** create GitHub Issues on wasi-webgpu, Wasmtime, or any other upstream. [wasi-webgpu#81](https://github.com/WebAssembly/wasi-webgpu/issues/81) was filed in error and is retracted.
