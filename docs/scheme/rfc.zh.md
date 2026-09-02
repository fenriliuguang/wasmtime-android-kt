# RFC：产品、GPU 宿主、gfx 循环

[English](rfc.md) | **中文**

与英文冲突时以英文为准。原 L5 / 生态 / 可插拔 GPU / gfx 循环四份 RFC 已合并为此页。

- 产品：Android 优先、可引用宿主；坐标 **`0.1.0`**（未发布前不因消费路径升 GAV）；永续 `0.x` 直至上游 1.0。
- GPU：默认进程内 Dawn C（`NativeGpu`）；`dawn-jni` 遗留；未接线 → `request-adapter` **`none`**。
- gfx：钉 `wasi-gfx:surface@0.2.0`；guest 拉 `on-frame`。剩余自动：尺寸/resize + 其余 pin 输入流。非紧急：`unconfigure`、带时间戳的 frame-event、Lost/Outdated `result`、多窗口。
- 收口：[`../agent/remaining.md`](../agent/remaining.md)。
