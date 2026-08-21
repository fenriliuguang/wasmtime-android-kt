# Guest 形状（`wasi:webgpu`）

[English](guest-shape.md) | **中文**

钉版 **`wasi:webgpu@0.3.0-rc.2`**（树内 [`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit)）。方法合格条件：名字 / `borrow` / 返回类型 / Guest 入参编组 / 真 `async` 均与 WIT 同构。禁止新切片以 host-fixed `u32` 验收。Agent 手册：[`../agent/webgpu-guest-semantics.md`](../agent/webgpu-guest-semantics.md)（剩余 optional 字段 + Dawn consume）。关闭队列：[`../agent/webgpu-guest-pipeline.md`](../agent/webgpu-guest-pipeline.md)。

S 系列与 DoD 见英文正文。未接线 / 无 adapter：`request-adapter` 返回 **`none`**。与英文冲突时以英文为准。
