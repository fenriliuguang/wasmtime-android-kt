# Agent 手册：立方体抖动重开（热路径阶段）

[English](gfx-hitch.md) | **中文**

P0 / P1 / `0.1.0` / native-dawn **consume** 自动刀已关闭。这次不是 consume 针。禁止重切那些旧队列。

consume 打空之后的**现行自动**剩余：把 NativeGpu / Dawn C 立方体的约 5 s **肉眼弹出**绑到热路径某一阶段，或绑到「弹出时进程内无冒尖」。针：[`../scheme/gfx-hitch.md`](../scheme/gfx-hitch.md)。节拍与阶段表：[`../mapping/gfx-hitch-native-dawn.zh.md`](../mapping/gfx-hitch-native-dawn.zh.md) **§6**。

**重开规则。** 映射 §§0–5 的 Closed / Likely / Mitigated 只作**档案**。本队列**不把它们当前提**。本分支 issue 300 已经选定：遗忘此前定位，从代码里的 vsync→present 一拍重开。

## 集成

分支 **`fix/300-gfx-cube-pop`**：**一刀一提交**。不要每刀开 PR。remaining 打空（或用户要求）再开/更新 issue-300 PR。

下一刀：`python3 ./scripts/gfx-hitch-remaining.py`。只做打印的 **Next:**（一次 commit）。

## 目标

Vivo V2458A、设置锁 120 Hz。仓外 `hosts/fullscreen-surface` + `GpuBackends.dawn()` 仍会肉眼弹出。对照 androidx 立方体不弹。用 `GfxHitch` 的 `hotpath` / `hotpath-spike` 把弹出绑上，再一刀具名后续。云上合成 120 Hz 节拍**关不掉**肉眼弹出。

## 本分支历史（档案，不要重切）

- `ad64463` — NativeGpu 立方体；去掉 androidx JNI 仍弹（D25）
- `b9aeb3d` — present 时间戳；P2–P5 / N9 关掉 guest / CM / SF 计数器 / 提交路径
- `94d5983` — **重开：**遗忘 Closed/Likely；按热路径阶段打 `Instant` 日志
- `9da8763` — 云上合成 120 Hz 1:1（关不掉肉眼弹出）

不要再把 D2/D3/N4 直方图当自动下一刀。HP-LOG 读 **`hotpath` / `hotpath-spike`**。

## 车道（自动序）

HP-RFC（本手册）→ **HP-LOG（真机 ≥2 min logcat）** → **HP-BIND（一句话绑定）** → 具名后续（合成器 SF 计数器已于 2026-09-02 重采、不拧旋钮；下一步是事件 `screenrecord` / guest·CM / fence）。

与英文冲突时以英文为准。在 HP-BIND 点名之前禁止再叠 keep / DisplayManager / GameState / 砍 JNI。禁止向上游 `gh issue create`。
