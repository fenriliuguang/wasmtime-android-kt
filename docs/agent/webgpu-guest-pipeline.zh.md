# Agent 手册：Guest compute / 3D 管线编组

[English](webgpu-guest-pipeline.md) | **中文**

形状挂名、默认 described L2、中期 first-cut（canvas A1–A4、S1–S3、`record-*`）之后用此页。选刀：`.\scripts\webgpu-guest-pipeline-remaining.ps1`（无 pwsh 则 `python3 ./scripts/webgpu-guest-pipeline-remaining.py`）打印的 **Next:**。一车道一 PR。

Dawn Kotlin 已能吃完整 bind-group / pipeline / pass；缺口在 Guest WIT → JNI。空 compute `submit` 与 `vertex_index` 离屏三角不算本队列验收。

- **自动序 P1→P5：** P1 `create-bind-group` 的 `entries`；P2 BGL **全部** entry（含 sampler/texture）；P3 render-pipeline 的 `vertex.buffers` + guest 片元 format；P4 begin-render-pass 的 depth + clear；P5 create-texture 的 mip/sampleCount/dimension
- **点名才做：** sampler/view 剩余字段、pipeline `constants`、S1–S3 剩余 descriptor、产品 canvas 上屏（禁止注册 `surface-*`）、Dawn **render** 引用切片（compute 引用已有）
- **禁止：** WebFetch WIT、无 offset 读 `cm.rs`/`jvm.rs`、改 hub 文件、crate `cargo fmt`、默认跑全量测试/仪器、wasi-gfx、CTS 宣称、**向上游提 GitHub issue**。除 P1 必要时可加 `HostArg::Longs` 外不加新 `HostArg` 变体。`request-adapter` 无后端仍返回 `none`；WIT `async` 保持真 async

复制源、白名单、窄测、JNI sentinel 以[英文正文](webgpu-guest-pipeline.md)为准。
