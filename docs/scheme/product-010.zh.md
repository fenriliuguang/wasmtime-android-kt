# `0.1.0` 产品队列（追踪）

[English](product-010.md) | **中文**

L5 **`0.1.0` 门禁** 的自动序。手册：[`../agent/product-010.md`](../agent/product-010.md)。与英文冲突时以英文为准。

下一刀：`python3 ./scripts/product-010-remaining.py`。针在英文正文；落地后删对应 `gap: p010 … pending`。完整帧循环尚未过闸：**P010-GFXB**（产品 `request-adapter`/`request-device`）→ **P010-GFXV**（Choreographer vsync）。P2 Wasmtime 钉是点名队列，不是本脚本 `Next:`。
