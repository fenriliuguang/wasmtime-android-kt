# Guest 形状（`wasi:webgpu`）

[English](guest-shape.md) | **中文**

钉版 **`wasi:webgpu@0.3.0-rc.2`**。**P0 已关闭**（[`../archive/p0-wasi-webgpu.zh.md`](../archive/p0-wasi-webgpu.zh.md)）。androidx 空洞：[`../mapping/gap-webgpu-wit-androidx.zh.md`](../mapping/gap-webgpu-wit-androidx.zh.md)。现行工作：[`../agent/wasi-p3.zh.md`](../agent/wasi-p3.zh.md)。

方法合格：名字 / `borrow` / 返回类型 / Guest 入参 / 真 `async` 与 WIT 同构。禁止新切片以 host-fixed `u32` 验收。未接线 `request-adapter` → **`none`**。与英文冲突时以英文为准。帧循环建议（非计划）：[`../mapping/frame-loop-suggestion.zh.md`](../mapping/frame-loop-suggestion.zh.md)。
