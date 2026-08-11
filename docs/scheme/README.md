# 方案索引（轨 B）

**中文** | [English](README.en.md)

本仓方案以章程为中心；根 README 给一句话与双轨表。

## 阶段

| 阶段 | 状态 |
|------|------|
| **文档立项 / 章程** | **完成**（2026-08-10） |
| **M0 构建骨架** | **完成**（2026-08-11） |
| **M1 同步 CM** | **完成**（2026-08-11） |
| **M2 真 CM async** | **完成**（2026-08-11） |
| **M3 接 L2** | **完成**（2026-08-11） |
| **M4 Android 上屏 smoke** | **完成**（Dawn clear→present；2026-08-11） |
| **M5 运行时硬化** | **完成**（错误 / 产物 / API / 贡献者壳 / WASI RFC；2026-08-11） |

## 文档

| 文档 | 说明 |
|------|------|
| [`charter.md`](charter.md) | 背景、愿景、堆叠、原则、风险 |
| [`dual-track.md`](dual-track.md) | 与轨 A 锁死 / 共享 / 隔离 |
| [`tech-stack.md`](tech-stack.md) | Wasmtime / JNI / NDK / 依赖 |
| [`milestones.md`](milestones.md) | M0–M5 DoD |
| [`api-stability.md`](api-stability.md) | experimental semver / 破坏性约定 |
| [`rfc-wasi-worlds.md`](rfc-wasi-worlds.md) | 更多 WASI world：有条件支持 / 优先级 |
| [`../contribute.md`](../contribute.md) | 贡献者构建；可选桌面开发壳 |
| [`non-goals.md`](non-goals.md) | 非目标硬表 |
| [`../mapping/threading-android.md`](../mapping/threading-android.md) | 线程契约 |

## 硬原则（摘录）

1. L2 不依赖 L1。  
2. Android-first；真 CM async 走官方 Wasmtime API。  
3. 不依赖 wasmtime4j 作运行时。  
4. 不阻塞、不替换轨 A sync-compat 主验收（直至独立 RFC）。  
5. experimental；不默认对外发布 / 合规宣称。  
