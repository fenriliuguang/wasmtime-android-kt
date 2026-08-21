# 路线图：`wasi:webgpu`（P0）

[English](roadmap-wasi-webgpu.md) | **中文**

钉版 `wasi:webgpu@0.3.0-rc.2`。形状见 [`guest-shape.md`](guest-shape.md)。GPU：本仓拥有 SPI，默认 Dawn bundle。禁止新开 host-fixed u32 功能 PR。上屏走提案 `gpu-canvas-context`；wasi-gfx 不升 P0。Guest 管线编组与 F1–F9 已关闭：[`../agent/webgpu-guest-pipeline.md`](../agent/webgpu-guest-pipeline.md)、[`../agent/webgpu-guest-semantics.md`](../agent/webgpu-guest-semantics.md)。Dawn consume + WG-6 剩余：[`../agent/webgpu-guest-dawn.md`](../agent/webgpu-guest-dawn.md)。

与英文冲突时以英文为准。
