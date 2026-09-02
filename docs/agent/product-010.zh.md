# Agent 手册：`0.1.0` 产品门禁

[English](product-010.md) | **中文**

P0 / P1 自动刀已关闭。本队列落地 L5 **`0.1.0` 产品子集** + gfx 帧循环。一车道一 PR。

`0.1.0` 针已空。下一刀走 [`gfx-hitch.md`](gfx-hitch.md)：`python3 ./scripts/gfx-hitch-remaining.py`。若点名 `P010-*`，仍用 `product-010-remaining.py`。只做打印的 **Next:**。

针在 [`../scheme/product-010.md`](../scheme/product-010.md)。P2 Wasmtime 钉是点名队列。完整帧循环已落地（**P010-GFXB** 产品 adapter/device + **P010-GFXV** Choreographer vsync）。最后 **P010-DEMO**：入口 README **链接仓外 demo**（引入即视为存在，不把 demo 做进本仓）+ 宣称表写一行真机上屏。P010-PUB 已落地 `publish.yml` 与坐标 `0.1.0`；secrets 缺失时不要强发。禁止 JS 式 rAF callback。禁止向上游 `gh issue create`。Cloud 无真机，仍要加仪器。

与英文冲突时以英文为准。
