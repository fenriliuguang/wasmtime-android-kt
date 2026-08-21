# Agent 手册：wasi:webgpu 形状刀

[English](webgpu-shape-slice.md) | **中文**

S6+ 产品 `[method]` 切片用此页。钉版 WIT 在树内，禁止下载上游。

- 选刀：用户已点名则照做，否则 `.\scripts\webgpu-shape-remaining.ps1`
- 禁止：WebFetch WIT、无 offset 读 `cm.rs`、横向扫第三份测试、改 hub 文件、crate `cargo fmt`、全量 native 测试、设备仪器、新 JNI、把 `gpu-canvas-context` 混进本手册（canvas 走 [`webgpu-midterm.md`](webgpu-midterm.md)）、改 README Transitional 长段、**向上游提 GitHub issue**
- 复制源、文件白名单、窄测命令以[英文正文](webgpu-shape-slice.md)为准
- 形状剩余为 0 时：默认 L2 走 [`webgpu-semantic-l2.md`](webgpu-semantic-l2.md)；canvas / S1–S3 / records / 真机走 [`webgpu-midterm.md`](webgpu-midterm.md)
