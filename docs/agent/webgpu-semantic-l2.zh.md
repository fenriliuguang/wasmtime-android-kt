# Agent 手册：wasi:webgpu 语义 L2

[English](webgpu-semantic-l2.md) | **中文**

形状挂接完成之后用此页。按 **调用方 resource** 分组，再按 **JNI 族**（标量 / borrow / list+result）拆 PR，每批约 2–4 个 `[method]`。本机仍逐条短测并 commit，一族完成再开 PR。

- 选刀：用户点了名字则只保留 **同一 JNI 族**；否则 `.\scripts\webgpu-semantic-l2-remaining.ps1`。host-fixed remaining 为 0 时改走 [`webgpu-midterm.md`](webgpu-midterm.md)，不要重切 sampler/label/limits
- `gpu-render-pass-encoder` 示例：A `draw`/`draw-indexed`；B `set-pipeline`/`set-vertex-buffer`/`set-index-buffer`；C `set-bind-group` 单独；`end` 可挂 A/B 末尾。禁止把 compute/bundle 同名方法、viewport/occlusion/debug 塞进同一 PR
- 禁止：WebFetch WIT、无 offset 读 `cm.rs`/`jvm.rs`、横向扫第三份模板、改 hub 文件、crate `cargo fmt`、全量 native 测试、设备仪器、在本手册加深 `gpu-canvas-context`/S1–S3/`record-*`、一条 PR 吞掉整个 resource、新增 `HostArg` 变体、**向上游提 GitHub issue**
- 金样：`create-buffer` + `exp_create_buffer_described` + `attachCreateBuffer`；guest `rep != 0` 时不要重建 adapter→pass
- 复制源、文件白名单、窄测命令以[英文正文](webgpu-semantic-l2.md)为准
