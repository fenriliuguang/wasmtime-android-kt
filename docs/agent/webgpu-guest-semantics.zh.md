# Agent 手册：剩余 descriptor 语义

[English](webgpu-guest-semantics.md) | **中文**

Guest 管线 P1–P5、sampler/view leftover、pipeline-constant JNI、S1–S3 leftover JNI、canvas 上屏、Dawn render cite **之后**用此页。关闭队列：[`webgpu-guest-pipeline.md`](webgpu-guest-pipeline.md)。选刀：`.\scripts\webgpu-guest-semantics-remaining.ps1`（无 pwsh 则 `python3 ./scripts/webgpu-guest-semantics-remaining.py`）打印的 **Next:**。一车道一 PR。

形状已挂、列表/布局/depth/mip 已进 host record；缺口是 JNI 仍丢的 **optional 字段**，或 Dawn 未把已 snapshot 的 Kotlin record 拷进 `androidx.webgpu`。空 compute `submit` 与 `vertex_index` 离屏三角不算本队列验收。测试用 `get-*` 构造器保持夹具，不要替换。

- **自动序 F1→F9：** F1 render-pipeline blend / multisample / cull / strip-index；F2 begin-render-pass **全部** color attachment；F3 create-texture `view-formats` + `label`；F4 create-buffer `mapped-at-creation` + `label`；F5 shader `label` + `compilation-hints`；F6 `xr-compatible`；F7 `default-queue`；F8 Dawn 吃 pipeline constants map；F9 Dawn 吃 `requiredLimits` map
- **点名才做：** `SupportedLimits` handle-0（禁止重切 limits first-cut）；`required-features` 全列表
- **禁止：** WebFetch WIT、无 offset 读 `cm.rs`/`jvm.rs`、改 hub 文件、crate `cargo fmt`、默认跑全量测试/仪器、wasi-gfx、CTS 宣称、新 `HostArg` 变体、**向上游提 GitHub issue**。不要重切 P1–P5 / sampler first-cut / canvas first-cut / constants map 资源 / S1–S3 JNI。`request-adapter` 无后端仍返回 `none`；WIT `async` 保持真 async

复制源、白名单、窄测、JNI / Dawn sentinel 以[英文正文](webgpu-guest-semantics.md)为准。
