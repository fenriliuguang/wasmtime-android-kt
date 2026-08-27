# RFC：wasi-gfx 帧循环（`0.1.0` 门禁）

**Status: Accepted (intent)** · 2026-08-26  
[English](rfc-wasi-gfx-frame-loop.md) | **中文**

英文为正文。

`0.1.0` 连续上屏走 **`wasi-gfx`**，不是 webgpu 里的 JS 式 callback。**不是**新 P0，不重开 G1–G9/WG-6。Guest **拉** `on-frame` stream；host 在 GpuThread 写 vsync。`run` 必须 async。钉 **`wasi-gfx:surface@0.2.0`**（P010-GFXP，`third_party/wasi-gfx/v0.2.0/`）。形状笔记：[`../mapping/frame-loop-suggestion.md`](../mapping/frame-loop-suggestion.md)。
