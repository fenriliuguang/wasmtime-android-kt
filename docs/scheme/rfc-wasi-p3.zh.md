# RFC：WASI 0.3 leftover 与「完整性」

[English](rfc-wasi-p3.md) | **中文**

**状态：Draft**（2026-09-05）。只讨论设计，本分支不写实现。与英文冲突时以英文为准。

产品政策仍以 [`rfc.md`](rfc.md) 为准（产品子集，不是 wasi-testsuite）。

- `0.1.2` 已有 cli/fs/TCP/HTTP **smoke**。named leftover：G-err / G-cmd / G-fs-full / G-sock-rest / G-http。
- NG-4 禁止把「全部 WASI 0.3 worlds」或完整 wasi-testsuite 当成唯一 KPI。加 `wasmtime-wasi` 仍须体积 + Android 线程说明。
- 选项：**A** 继续 named-only，等外仓 example 撞墙再切；**B**（草案倾向）thin-host 长刀队列 `cursor/wasi-p3-leftover-b677` 补齐 G-*，**不**宣称完整 0.3；**C** 换 `wasmtime-wasi`（另 RFC + 数字）；**D** testsuite 当 DoD（拒绝）。
- B 仍 Out：testsuite 门槛、`wasmtime-wasi`、0.2 pollable、timezone、guest 线程、基准测试、本仓 1.0。
- 接受本 RFC 后才改 [`rfc.md`](rfc.md) §4：NG-4 保留；leftover 从「永不自动切」改为 living queue。
