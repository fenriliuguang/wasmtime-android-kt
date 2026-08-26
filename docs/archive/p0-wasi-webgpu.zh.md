# P0 收口：`wasi:webgpu`（归档）

[English](p0-wasi-webgpu.md) | **中文**

> **2026-08-22 关闭**（`main` `0ed028b`，PR #253）。不要再开 wasi:webgpu 实现队列。活差距表：[`../mapping/gap-webgpu-wit-androidx.zh.md`](../mapping/gap-webgpu-wit-androidx.zh.md)。现行工作：[`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。P1 收口：[`p1-wasi-p3.zh.md`](p1-wasi-p3.zh.md)。

钉版 `wasi:webgpu@0.3.0-rc.2`。形状门禁仍在 [`../scheme/guest-shape.md`](../scheme/guest-shape.md)。与英文冲突时以英文为准。

## 现状

WG-1…WG-6、S 系列挂名、F1–F9、G1–G9 均已合入。不宣称 CTS / 合规产品。wasi-gfx 不是 P0。

## 时间线

2026-08-11 薄 L1 → 08-16 挂名后冻结 host-fixed `u32` 并完成 S1–S5 → 08-17 Dawn bundle → 08-20 L2 described JNI 抽空 lift-only → 08-21 F/G 车道 → 08-22 WG-6 真 guest 切片与本收口。

## 留下的问题

- 新切片不得以 host-fixed `u32` 验收（NG-12）。
- 禁止向上游提 GitHub issue（wasi-webgpu#81 误提已撤回）。
- Android：`opt-level=0` 导致 stream.write 仪器 SIGSEGV；Android 16 后台 `startActivity` 到不了 `RESUMED`；非 Vulkan 会 `WINDOW_IN_USE`。
- androidx 空洞（compilation-hints、canvas color-space/tone-mapping）见差距表，不要重切 G 车道。
