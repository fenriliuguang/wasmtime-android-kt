# RFC：产品、GPU 宿主、gfx 循环

[English](rfc.md) | **中文**

与英文冲突时以英文为准。原 L5 / 生态 / 可插拔 GPU / gfx 循环四份 RFC 已合并为此页。

- 产品：Android 优先、可引用宿主；坐标 **`0.1.2-SNAPSHOT`**（允许 SNAPSHOT）；后续升 GAV 见 [`api-stability.md`](api-stability.md)；永续 `0.x` 直至上游 1.0。
- GPU：默认进程内 Dawn C（`NativeGpu`）；**0.1.2-SNAPSHOT** Maven `host-dawn` 打进 `--prebuilt` `libwebgpu_dawn.so`（不要用 `0.1.1` 跑 GPU）；`dawn-jni` 遗留；未接线 → `request-adapter` **`none`**。
- gfx：钉 `wasi-gfx:surface@0.2.0`；guest 拉 `on-frame`。尺寸/resize 与 pin 输入流已落地。非紧急：`unconfigure`、带时间戳的 frame-event、Lost/Outdated `result`、多窗口。
