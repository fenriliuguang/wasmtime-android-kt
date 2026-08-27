# Agent 手册：`0.1.0` 产品门禁

[English](product-010.md) | **中文**

P0 / P1 自动刀已关闭。本队列落地 L5 **`0.1.0` 产品子集** + gfx 帧循环。一车道一 PR。

下一刀：`python3 ./scripts/product-010-remaining.py`（或 `.\scripts\product-010-remaining.ps1`）。只做打印的 **Next:**。

针在 [`../scheme/product-010.md`](../scheme/product-010.md)。P2 Wasmtime 钉是点名队列。完整帧循环：**P010-GFXB** 然后 **P010-GFXV**（两帧预缓冲不够）。P010-PUB 已落地 `publish.yml` 与坐标 `0.1.0`；secrets 缺失时不要强发。禁止 JS 式 rAF callback。禁止向上游 `gh issue create`。Cloud 无真机，仍要加仪器。

与英文冲突时以英文为准。
