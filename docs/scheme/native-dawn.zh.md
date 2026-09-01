# Native Dawn host 队列（追踪）

[English](native-dawn.md) | **中文**

默认 `wasi:webgpu` 消费改走 **Dawn C** 的自动序。手册：[`../agent/native-dawn.md`](../agent/native-dawn.md)。与英文冲突时以英文为准。

长期分支 **`cursor/native-dawn-rewrite-1355`**。下一刀：`python3 ./scripts/native-dawn-remaining.py`（一刀一 **commit**，不要每刀开 PR）。针在英文正文；落地后删对应 `gap: nd … pending`。全部针删完再提 **一个** PR 合 `main`。立方体不是 consume 针。
