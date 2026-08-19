# Agent 手册：wasi:webgpu 语义 L2

[English](webgpu-semantic-l2.md) | **中文**

形状挂接完成之后用此页。每刀把 **一个** 已挂 `[method]` 的 Guest 字段经 JNI 送到现有 Kotlin/Dawn host。

- 选刀：用户已点名则照做，否则 `.\scripts\webgpu-semantic-l2-remaining.ps1`；默认优先 host-fixed 列表；未点名时第一刀 **`gpu-device.create-sampler`**
- 禁止：WebFetch WIT、无 offset 读 `cm.rs`/`jvm.rs`、横向扫第三份模板、改 hub 文件、crate `cargo fmt`、全量 native 测试、设备仪器、混进 `gpu-canvas-context`、一 PR 多方法、扩展 `HostArg` 字符串/字节（除非用户点了带 string 的方法）
- 金样：`create-buffer` + `exp_create_buffer_described` + `attachCreateBuffer`
- 复制源、文件白名单、窄测命令以[英文正文](webgpu-semantic-l2.md)为准
