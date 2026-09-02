# Agent 手册：立方体抖动重开（热路径阶段）

[English](gfx-hitch.md) | **中文**

P0 / P1 / `0.1.0` / native-dawn **consume** 自动刀已关闭。这次不是 consume 针。禁止重切那些旧队列。

consume 打空之后的**现行自动**剩余：**空**（肉眼弹出绑到仓外 guest `sincos`；映射 §6.9）。针：[`../scheme/gfx-hitch.md`](../scheme/gfx-hitch.md)。节拍：[`../mapping/gfx-hitch-native-dawn.zh.md`](../mapping/gfx-hitch-native-dawn.zh.md) **§6**。

**重开规则。** 映射 §§0–5 的 Closed / Likely / Mitigated 只作**档案**。本队列**不把它们当前提**。本分支 issue 300 已经选定：遗忘此前定位，从代码里的 vsync→present 一拍重开。

## 集成

分支 **`fix/300-gfx-cube-pop`**：**一刀一提交**。不要每刀开 PR。remaining 打空（或用户要求）再开/更新 issue-300 PR。

下一刀：`python3 ./scripts/gfx-hitch-remaining.py`。只做打印的 **Next:**（一次 commit）。

## 目标

Vivo V2458A、设置锁 120 Hz。仓外旋转立方体约 5 s **肉眼弹出已收口**为 guest 泰勒/`wrap_pi`（以及后来共用欧拉角的 `fold_pi` 跳变）。不是 NativeGpu / Dawn C present。androidx 对照从不弹。

## 本分支历史（档案，不要重切）

- `ad64463` — NativeGpu 立方体；去掉 androidx JNI 仍弹（D25）
- `b9aeb3d` — present 时间戳；P2–P5 / N9 关掉 guest / CM / SF 计数器 / 提交路径
- `94d5983` — **重开：**遗忘 Closed/Likely；按热路径阶段打 `Instant` 日志
- `9da8763` — 云上合成 120 Hz 1:1（关不掉肉眼弹出）

不要再把 D2/D3/N4 直方图当自动下一刀。HP-LOG 读 **`hotpath` / `hotpath-spike`**。

## 车道（自动序）

HP-RFC（本手册）→ **HP-LOG** → **HP-BIND** → 具名后续（合成器 SF 已采、不拧；**肉眼弹出 §6.9 收口为 guest sincos**）。

与英文冲突时以英文为准。在 HP-BIND 点名之前禁止再叠 keep / DisplayManager / GameState / 砍 JNI。禁止向上游 `gh issue create`。
