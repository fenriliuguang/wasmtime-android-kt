# Agent 手册：Dawn consume + WG-6 剩余

> **已于 2026-08-22 归档。** 不要按本页实现。P0 收口：[`p0-wasi-webgpu.zh.md`](p0-wasi-webgpu.zh.md)。差距：[`../mapping/gap-webgpu-wit-androidx.zh.md`](../mapping/gap-webgpu-wit-androidx.zh.md)。现行队列：[`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。

[English](webgpu-guest-dawn.md) | **中文**

Guest 管线 P1–P5 与 leftover descriptor F1–F9（含 handle-0 / required-features 全列表）**之后**用此页。关闭队列：[`webgpu-guest-pipeline.md`](webgpu-guest-pipeline.md)、[`webgpu-guest-semantics.md`](webgpu-guest-semantics.md)。选刀：`.\scripts\webgpu-guest-dawn-remaining.ps1`（无 pwsh 则 `python3 ./scripts/webgpu-guest-dawn-remaining.py`）打印的 **Next:**。一车道一 PR。

JNI 已把 leftover optional 字段打进 Kotlin record；缺口是 Dawn 未拷进 `androidx.webgpu`、少数 WIT 字段从未 snapshot、`layout: auto` 抛错，以及 WG-6 仍缺 **guest 画出来的** compute / 3D / 上屏切片。空 compute `submit`、1×1 清屏 cite、`vertex_index` 离屏三角、host 清屏上屏不算本队列验收。测试用 `get-*` 构造器保持夹具，不要替换。

- **自动序 G1→G9：** G1 Dawn 吃 render-pipeline blend / cull / MSAA；G2 texture `view-formats`；G3 shader compilation-hints；G4 `xr-compatible`；G5 `default-queue`；G6 color `write-mask`；G7 depth-stencil leftover（stencil / bias）；G8 canvas configure 剩余字段；G9 `layout: auto`
- **点名才做：** WG-6 真 guest compute（BGL + bind-group + dispatch，非 empty pass）；WG-6 真 3D（vertex + draw，非 1×1 清屏）；WG-6 上屏 guest 画出的帧（非 host 清屏）；F9 跳过的 stage-only storage limit keys
- **禁止：** WebFetch WIT、无 offset 读 `cm.rs`/`jvm.rs`、改 hub 文件、crate `cargo fmt`、默认跑全量测试/仪器、wasi-gfx、CTS 宣称、新 `HostArg` 变体、**向上游提 GitHub issue**。不要重切 P1–P5 / F1–F9 / sampler first-cut / canvas first-cut / cite 切片。`request-adapter` 无后端仍返回 `none`；WIT `async` 保持真 async

复制源、白名单、窄测、Dawn / JNI sentinel 以[英文正文](webgpu-guest-dawn.md)为准。
