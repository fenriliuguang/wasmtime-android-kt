# Agent 手册：wasi:webgpu 中期四车道

[English](webgpu-midterm.md) | **中文**

默认形状剩余为 0（不含 canvas）且默认语义 L2 host-fixed 为 0 之后用此页。选刀：`.\scripts\webgpu-midterm-remaining.ps1`（无 pwsh 则 `python3 ./scripts/webgpu-midterm-remaining.py`）打印的 **Next:**。一车道一 PR；形状挂名与 L2 不混刀。

- **A WG-6 canvas：** A1 四方法形状 + 测试专用 `get-canvas-context`；A2 `configure`（`device.rep`+format）；A3 `get-current-texture`（`unconfigure` 可挂）；A4 `get-configuration`。禁止把 experimental `surface-*` 注册成产品 WIT
- **B S1–S3：** B1 `queue` 用 `device.rep` 禁止重建 adapter；B2 `request-device` 先不传 limits map / 可跳过 label 字符串；B3 `request-adapter` 先 `power-preference` + `force-fallback-adapter`。保持真 async
- **C `record-*`：** 默认不做。用户点名或 pipeline constants 需要时：一 resource 一 PR，mutate 与 iterate 拆开
- **D 真机/WG-5：** 用户点名才做。不新挂 WIT；Dawn 一条 render 或 compute。Android 新信息只写本仓 changelog / mapping。**禁止**向 wasi-webgpu / Wasmtime 或任何上游开 GitHub issue。无 CTS 宣称

禁止：WebFetch WIT、无 offset 读 `cm.rs`/`jvm.rs`、改 hub 文件、crate `cargo fmt`、A–C 跑全量测试/仪器、wasi-gfx、新 `HostArg` 变体、**向上游提 issue**。复制源、白名单、窄测以[英文正文](webgpu-midterm.md)为准。
