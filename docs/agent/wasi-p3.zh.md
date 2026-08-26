# Agent 手册：WASI 0.3（P1）

[English](wasi-p3.md) | **中文**

P0 `wasi:webgpu` **已关闭**。收口：[`../archive/p0-wasi-webgpu.zh.md`](../archive/p0-wasi-webgpu.zh.md)。WebGPU 空洞：[`../mapping/gap-webgpu-wit-androidx.zh.md`](../mapping/gap-webgpu-wit-androidx.zh.md)。不要重切 G1–G9 / F1–F9 / guest-pipeline / WG-6。

本队列把 **已批准 WASI 0.3** 收到官方 WIT（告别过渡 `u64` / `future<u32>`），且 **每刀真机仪器**。选刀：`.\scripts\wasi-p3-remaining.ps1`（无 pwsh 则 `python3 ./scripts/wasi-p3-remaining.py`）打印的 **Next:**。一车道一 PR。

- **自动序 W1→W8：** 已合入 smoke。其后官方形状短刀见 [`../mapping/gap-wasi-p3-wit.zh.md`](../mapping/gap-wasi-p3-wit.zh.md)（P1-FS1…P1-HT1）。不要重切 W1–W8。
- **点名才做：** 0.2 polyfill、全量 wasi-testsuite、默认启用 `wasmtime-wasi` crate
- **禁止：** 再开 WebGPU/Dawn 刀、无审查加 `wasmtime-wasi`、无 offset 读整份 `cm.rs`、改枢纽文件、Latch 假异步、`wasi:io@0.2` 主路径、**向上游提 GitHub issue**

复制源、白名单、窄测以[英文正文](wasi-p3.md)为准。
