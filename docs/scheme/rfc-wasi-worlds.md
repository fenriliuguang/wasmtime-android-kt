# RFC：是否 / 如何支持更多 WASI world

**状态：Superseded（2026-08-11）** · 原 Accepted · M5  
**中文** | （暂无 EN）

> **已被取代：** [`long-term-plan.md`](long-term-plan.md) + [`wasi-p3-surface.md`](wasi-p3-surface.md) + [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md)。  
> 短期口径（「标准 WASI 默认不做 / webgpu 仅 experimental」）在长期阶段已修订为：**主推 WASI 0.3 正式面**，提案面 **优先 wasi:webgpu**。  
> 下文保留为 M5 收口史实；新排期以长期计划为准。

## 1. 背景

轨 B 短期已验证：

- 官方 Wasmtime + 自研 JNI 薄 L1  
- 真 CM async（M2）  
- 接轨 A experimental webgpu Host（M3–M4）

章程愿景允许「托管一类 WASI / 自定义 world」，但 [`non-goals.md`](non-goals.md) 明确：

- **NG-4**：不以完整 WASI Preview3 / `wasi-http` / `wasi-nn` 为短期关门  
- **NG-9**：不以 wasi-gfx / 多 window 为短期目标  
- **DG-3**：完整 WASI 云/CLI 运行时 → 产品化后再议  

需要书面回答：**还要不要更多 world？按什么顺序？什么时候开做？**

## 2. 决策摘要

| 问题 | 决定 |
|------|------|
| 是否支持更多 WASI world？ | **有条件支持**：按需、切片化；**不**做「完整 WASI 套件」产品承诺 |
| 近端（M5 收口后 ~ 下一主线）优先？ | **深化现有 experimental webgpu-cm 子集**（追齐轨 A cube 差距），而非新开 WASI 标准 world |
| 标准 WASI（cli / http / clocks / random / io …）？ | **默认不做**；仅当有明确 Guest 工具链或演示阻塞时，允许**极小 stub**（见 non-goals「最小 WASI 子集」） |
| wasi:webgpu 合规世界？ | **不做**合规宣称；继续走 experimental 坐标，跟轨 A L2 |
| Preview3 / wasi-http / wasi-nn / wasi-gfx？ | **维持 NG-4 / NG-9**；另开 RFC 才能升格为里程碑 |

## 3. 分层模型

```text
L0  运行时能力（本仓主责）
    Engine / Store / Linker / Component / Instance
    sync host · CM async · resource u32 · 错误模型 · ABI 产物

L1  Host world 胶水（本仓薄注册 + 轨 A / 外部 L2）
    今日：experimental:webgpu-cm@0.8.0 子集
    未来：仅当 L0 稳定且有 Guest 需求时再加 world

L2  设备 / 协议实现（优先复用轨 A；本仓不重造 Dawn）
```

原则：

1. **World 不是 runtime 的 KPI**——多挂一个 WASI package 不构成 M5+ 成功标准。  
2. **L2 不依赖 L1** 不变；新 world 不得把设备逻辑塞进 JNI。  
3. 新 world 默认 **experimental**，走 [`api-stability.md`](api-stability.md) 最不稳定层。

## 4. 优先级（路线图）

### P0 — 当前已承诺 / 继续做

| World / 面 | 动作 |
|------------|------|
| `experimental:webgpu-cm`（扁平子集） | 按 [`../mapping/gap-m4-vs-cube.md`](../mapping/gap-m4-vs-cube.md) 缩小与轨 A cube 差距；**不**宣布替换轨 A 主验收 |

### P1 — 有证据再开切片

| 候选 | 开做条件（须同时满足） |
|------|------------------------|
| 额外 CM host import（非 WASI 标准名） | 具体 Guest 阻塞 + 可测仪器/JVM 用例 |
| 极小 WASI stub（如 `wasi:io` / clocks 只读子集） | Guest 工具链强制依赖；stub **不作**产品卖点；写清「非合规」 |
| 更多 async WIT 形状（stream 等） | M2 闸门模式可复用；有独立 DoD |

### P2 — 另开 RFC 才能进里程碑

| 候选 | 说明 |
|------|------|
| 完整 `wasi:cli` / Preview2 云运行时 | DG-3 |
| `wasi-http` / `wasi-nn` | NG-4 |
| `wasi-gfx` / 多 window | NG-9 |
| 标准 `wasi:webgpu` 合规认证 | NG-5 |

## 5. 准入检查清单（任何新 world PR）

1. **动机**：哪个 Guest / 演示被阻塞？能否用现有 experimental 子集解决？  
2. **归属**：实现落在轨 A L2 还是本仓 Kotlin 回调？禁止在 Rust JNI 堆业务。  
3. **异步**：若含 `async func`，必须走官方 concurrent API（禁止 sync-compat 冒充）。  
4. **测试**：至少一条可复现路径（优先 Android 仪器；桌面 JVM 可作辅）。  
5. **文档**：更新差距清单或本 RFC 附录；CHANGELOG；必要时升 `0.MINOR`。  
6. **非目标**：确认未触碰 NG-1（替换轨 A）、NG-4/9（未经 RFC 升格）。

## 6. 明确不承诺

- 不提供「WASIness 完成度」仪表盘式路线图  
- 不与 Wasmtime 上游 WASI 特性表逐项对齐作为关门条件  
- 不因「别的 runtime 支持了 X」而自动排期 X  

## 7. 修订

- 小修订（措辞 / 链接）：直接 PR + CHANGELOG Docs。  
- 改变 P0/P1/P2 归属或推翻 §2 决策：新开 RFC，废止或标注本文件 `Superseded`。  
