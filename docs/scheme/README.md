# 方案索引（轨 B）

**中文** | [English](README.en.md)

本仓方案以章程为中心；根 README 给一句话与双轨表。  
**现行主线：** [`long-term-plan.md`](long-term-plan.md)（WASI 0.3 · wasi:webgpu · 官方 Wasmtime）。

## 阶段

| 阶段 | 状态 |
|------|------|
| **文档立项 / 章程** | **完成**（2026-08-10） |
| **短期 M0–M5 薄 L1** | **已归档**（2026-08-11）→ [`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md) |
| **长期计划（文档期）** | **现行**（2026-08-11）→ [`long-term-plan.md`](long-term-plan.md) |
| **长期 L1+ 代码切片** | **进行中**（stream · random · clocks · cli stdout · webgpu W1/W2 `request-adapter` 真 async；下一刀 `request-device` async / W3 / cli stdin） |

## 文档

### 现行规划

| 文档 | 说明 |
|------|------|
| [`long-term-plan.md`](long-term-plan.md) | 长期战略；L0–L5；P0/P1/P2 硬序 |
| [`wasi-p3-surface.md`](wasi-p3-surface.md) | WASI 0.3 正式特性优先级与切片门禁 |
| [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) | wasi:webgpu 提案推进（P0） |
| [`w1-dual-register.md`](w1-dual-register.md) | W1：提案名双注册（已交付；过渡扁平；下一刀 W2） |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | 官方 Wasmtime 钉版 / 升级 / 回归 |
| [`vcs-workflow.md`](vcs-workflow.md) | 短命分支 + PR；Ruleset / CI 清单 |
| [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) | 贡献入口（权限 / CI / PR） |

### 章程与契约

| 文档 | 说明 |
|------|------|
| [`charter.md`](charter.md) | 背景、愿景、原则、风险 |
| [`dual-track.md`](dual-track.md) | 与轨 A 锁死 / 共享 / 隔离 |
| [`tech-stack.md`](tech-stack.md) | Wasmtime / JNI / NDK / 依赖 |
| [`api-stability.md`](api-stability.md) | experimental semver / 破坏性约定 |
| [`non-goals.md`](non-goals.md) | 非目标硬表（长期修订） |
| [`../contribute.md`](../contribute.md) | 贡献者构建；可选桌面开发壳 |
| [`../mapping/threading-android.md`](../mapping/threading-android.md) | 线程契约 |
| [`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md) | W0：experimental ↔ 提案对照 |

### 归档 / 史实

| 文档 | 说明 |
|------|------|
| [`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md) | 短期阶段归档入口 |
| [`milestones.md`](milestones.md) | M0–M5 DoD **冻结史实** |
| [`rfc-wasi-worlds.md`](rfc-wasi-worlds.md) | **Superseded**（M5 旧 worlds 口径） |

## 硬原则（摘录）

1. L2 不依赖 L1；不重造 Dawn。  
2. Android-first；真 CM async / WASI 0.3 异步走官方 Wasmtime API。  
3. 不依赖 wasmtime4j；追踪官方 Wasmtime。  
4. 主推 WASI 0.3 **已批准切片** + **wasi:webgpu 提案**；不作全量套件 / 合规空喊。  
5. 不阻塞、不替换轨 A sync-compat 主验收（直至独立 RFC）。  
6. experimental；不默认对外发布。  
