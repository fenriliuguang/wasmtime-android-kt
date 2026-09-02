# Agent 手册：native Dawn host（全量钉）

[English](native-dawn.md) | **中文**

P0 **形状**与 P1、`0.1.0` 自动刀已关闭。本队列在**同一钉**后面把默认消费从 JNI/androidx 换成 **进程内 Dawn C**。禁止重切 G1–G9 / F1–F9 / WG-6 **那些旧队列名**。

## 集成（仅本队列）

长期分支 **`cursor/native-dawn-rewrite-1355`**：**一刀一提交**，**整段做完再提一个 PR** 合入 `main`。不要每刀开 PR。待在该分支上推送提交。`native-dawn-remaining.py` 打空（**ND-DEVICE** 落地）之后才开 PR，并保留各刀提交（不要 squash 成一颗）。

下一刀：`python3 ./scripts/native-dawn-remaining.py`。只做打印的 **Next:**（一次 commit）。针在 [`../scheme/native-dawn.md`](../scheme/native-dawn.md)。

consume 已打空（`ND-DEVICE` / `#299`）。立方体抖动**不是** consume 针。现行剩余：**空**。节拍同步：[`../mapping/gfx-hitch-checklist.zh.md`](../mapping/gfx-hitch-checklist.zh.md)。

## 目标

默认产品路径：guest 仍见 `wasi:webgpu@0.3.0-rc.2` + `wasi-gfx`；Kotlin `Store` / `Linker` / `WebGpuBackend` 仍是壳与 BYO；热路径 **不**再 `ExperimentalHostCallbacks` → androidx JNI。

**产品目标是全量钉能力**（224 个 `[method]` 在默认后端落到 Dawn C）。仓外立方体只是 **演示 / 上屏证据**，不能当 consume 车道 DoD，也不能用来跳过 **ND-REST**。

## 复用

留下 `cm.rs` lowering、`fixtures/w1`、`wasi_webgpu_method`、现有仪器、`GfxOnFrameGate`。`DawnWasiWebGpuHost.kt` 与 hitch 清单当**映射和禁令**，不要把 278 个 `exp_*` 再译成 Rust。

## 车道（自动序）

ND-DISP → ND-SO → ND-HOST → ND-BOOT → ND-RES → ND-PIPE → ND-ENC → ND-QUEUE → **ND-REST（全量 method 测试）** → ND-SURF → ND-DEFAULT → ND-CLAIM → ND-DEVICE（仪器 + 立方体演示行）→ **一个 PR**。

与英文冲突时以英文为准。禁止 JS 式 callback。禁止双份 Dawn `.so`。禁止向上游 `gh issue create`。Cloud 无真机，仍要列出仪器。
