# 长期计划：WASI 0.3 · wasi:webgpu · 官方 Wasmtime

**中文** | （暂无 EN）

> **状态：现行规划（文档期，2026-08-11）。** 本阶段 **不**要求立刻改代码。  
> 短期薄 L1 已归档：[`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md)。  
> 配套：[`wasi-p3-surface.md`](wasi-p3-surface.md) · [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) · [`wasmtime-tracking.md`](wasmtime-tracking.md)。

## 1. 转向一句话

从「证明 Android 上能跑官方 Wasmtime 真 CM async + 插轨 A L2」  
→ 建设 **Android-first JVM Component runtime**，以：

1. **WASI 0.3（WASI P3）已批准正式面** 为主推能力底座；  
2. **`wasi:webgpu` 提案** 为首发场景与提案推进优先项（本仓存在的核心触发点）；  
3. **官方 Wasmtime** 为唯一引擎依赖，建立可追踪的版本与特性跟进机制。

仍为 **experimental**；**不**默认对外发布；**不**静默替换轨 A 主验收（见 [`dual-track.md`](dual-track.md)）。

## 2. 战略优先级（硬序）

```text
P0  wasi:webgpu 提案推进（CM async 原生语义 + Android Host 路径）
    ↑ 依赖
P1  WASI 0.3 正式原语与核心 world 子集（runtime 可承载）
    ↑ 依赖
P2  官方 Wasmtime 版本 / feature 追踪与升级闸门
    ↑ 底座
已归档  M0–M5 薄 L1（同步 CM · 真 async · experimental L2 smoke）
```

解释：

| 级 | 含义 |
|----|------|
| **P0** | 产品叙事与工程排期的第一公民；缺口优先排期；对外谈本仓时默认指这条线 |
| **P1** | 为 P0 与通用 Guest 提供 **已批准** 的 WASI 0.3 能力；按 [`wasi-p3-surface.md`](wasi-p3-surface.md) 切片，**不做**「完整 WASI 套件」口号式 KPI |
| **P2** | 工程卫生：钉版本、跟 changelog、升级 RFC；Wasmtime 不是产品卖点本身，但是 **不可替代依赖** |

提案未批准部分（如 `wasi:webgpu` 仍 Phase 2）**可以**主推实现与反馈，但宣称必须写清阶段，禁止「合规标准产品」话术（继承 NG-5 精神）。

## 3. 与短期归档的关系

| 短期（已归档） | 长期（现行） |
|----------------|--------------|
| 假 world / experimental 扁平 webgpu-cm | 对齐 / 收敛到提案 `wasi:webgpu` WIT（见专章） |
| 单条 future smoke（M2） | WASI 0.3：`async func` + `future` + **`stream`** 一等支持 |
| 接轨 A L2 smoke（M3–M4） | 继续 **复用** 轨 A L2 / Dawn；推动 L2 与提案 WIT 对齐，而非重造第二套 Dawn |
| 「更多 world 有条件再说」（旧 RFC） | **主推已批准 P3 面**；提案面以 webgpu 为先 |
| M0–M5 编号 | 新堆叠 **L0–L5**（下节）；旧号冻结 |

旧 RFC [`rfc-wasi-worlds.md`](rfc-wasi-worlds.md) 标为 **Superseded**。

## 4. 目标堆叠（L0–L5）

```text
L0  底座冻结与追踪机制
    归档 M0–M5；Wasmtime 钉版与跟踪表；文档索引切换

L1  WASI 0.3 原语完备（相对本仓 JNI/Kotlin 面）
    async func · future（已有）· stream · run_concurrent 泵产品化
    见 wasi-p3-surface.md §「原语层」

L2  WASI 0.3 核心 imports 子集（按需切片）
    clocks / random → cli stdio 流模式 → 其余按 Guest 阻塞开门
    见 wasi-p3-surface.md §「world / package」

L3  wasi:webgpu 提案主链（P0）
    标准 WIT 坐标（钉提案版本）· 真 async 方法 · 经 L1→轨 A L2
    Android 仪器：adapter/device 主链 → 渲染 / compute 切片
    见 roadmap-wasi-webgpu.md

L4  双轨可选合流准备
    与轨 A cube / 提案 Guest 差距清单收敛；切换 RFC 草案（不自动切主验收）

L5  运行时产品化门槛
    API 冻结候选、多 ABI CI、错误/线程契约稳定、是否对外发布的单独 RFC
```

**硬序：** L0 文档先落地（本批）→ L1 原语未达标前 **不**承诺大型 WASI world → L3 可与 L1/L2 部分并行，但 **webgpu 的 async WIT 不得回退 sync-compat**。

### 4.1 阶段性成功标准

| 阶段 | 成功长什么样 |
|------|----------------|
| 近端 | 文档与追踪机制就位；Wasmtime 跟踪表可回答「我们钉哪、缺啥」 |
| 中端 | JNI/Kotlin 面能承载 WASI 0.3 `stream` + 至少一条正式 package 子集；`wasi:webgpu` 提案主链在 Android 上真 async 跑通（非仅 experimental 扁平名） |
| 远端 | 第三方以「Android 上跟 Wasmtime / WASI 0.3、优先 wasi:webgpu」理解本仓；轨 A 切换 L1 具备可评审 RFC |

## 5. 架构原则（继承 + 修订）

继承 [`charter.md`](charter.md) §4，并强调：

1. **L2 不依赖 L1** — Host / Dawn 仍优先轨 A；本仓不实现第二套 Dawn（NG-7）。  
2. **官方语义优先** — WASI 0.3 / CM async 走 Wasmtime 上游；禁止再发明 pollable 时代的兼容层当「真异步」。  
3. **提案推进 ≠ 合规宣称** — 可为 `wasi:webgpu` 提反馈、对齐 WIT、跑 CTS 子集；未达标前不宣传合规产品。  
4. **P3 正式面主推、提案切片化** — 已批准 package 按表面文档优先级；提案只把 webgpu 放 P0，其它提案默认旁路。  
5. **Android-first** — 桌面壳仅开发便利；门禁以真机为准。  
6. **双轨隔离** — 直至独立合流 RFC，轨 A sync-compat 主验收不变。

## 6. 非目标（长期阶段修订摘要）

完整表见 [`non-goals.md`](non-goals.md)。相对短期的关键变化：

| 原短期口径 | 长期口径 |
|------------|----------|
| NG-4：不以完整 P3 为**短期关门** | **主推** WASI 0.3 **已批准**特性；仍 **不**以「实现全部 P3 worlds / 过全量 wasi-testsuite」为单一 KPI |
| rfc-wasi-worlds：标准 WASI 默认不做 | 正式 P3 package **按优先级切片做**；未批准提案除 webgpu 外默认不做 |
| wasi:webgpu 合规 = P2 另开 RFC | **实现与提案跟进升为 P0**；**合规认证 / 产品宣称**仍另开门槛 |

仍绝对禁止：静默替换轨 A、依赖 wasmtime4j、重造完整 Kotlin WebGPU 客户端 API、默认 Central 发布、用 sync-compat 冒充实 async。

## 7. 文档地图（现行）

| 文档 | 角色 |
|------|------|
| **本页** | 长期战略与 L0–L5 堆叠 |
| [`wasi-p3-surface.md`](wasi-p3-surface.md) | WASI 0.3 正式特性：优先级与切片门禁 |
| [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) | wasi:webgpu 提案推进路线 |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | 官方 Wasmtime 依赖追踪与升级流程 |
| [`vcs-workflow.md`](vcs-workflow.md) | 短命分支 + PR；并行矩阵；开源接 PR |
| [`archive/m0-m5-thin-l1.md`](archive/m0-m5-thin-l1.md) | 短期阶段归档 |
| [`milestones.md`](milestones.md) | M0–M5 **冻结史实** |
| [`dual-track.md`](dual-track.md) | 与轨 A 契约（仍有效） |
| [`api-stability.md`](api-stability.md) | `0.x-experimental`（仍有效） |

### 7.1 并行与分支（摘要）

战略多线 **不**等于多条长期 git 分支。详见 [`vcs-workflow.md`](vcs-workflow.md)：

- 默认：`main` + 短命 `docs/`·`feat/` 分支 + PR  
- **不**建常驻 `feature/stream` / `feature/webgpu` 第二主干  
- 可同时开少量短 PR（文档 ‖ webgpu W0–W1 ‖ clocks/random）；**stream 单线优先合入**后再开 stdio  

## 8. 近端文档期 DoD（本批，无代码）

- [x] 短期 M0–M5 归档页  
- [x] 本长期计划  
- [x] WASI P3 表面优先级  
- [x] wasi:webgpu 提案路线图  
- [x] Wasmtime 追踪机制  
- [x] 索引 / 章程 / non-goals / 旧 RFC 状态 / 根 README / CHANGELOG 对齐  
- [x] 版本控制 / 短命分支工作流（[`vcs-workflow.md`](vcs-workflow.md)）  

代码期启动条件：本批文档经短命 PR 合入 `main` 后，另开 `feat/…` 实现切片（建议首刀：**L1 stream 面** 或 **wasi:webgpu WIT 钉版差距表**；文档可并行、依赖 stream 的落地须串行）。

## 9. 修订

- 小修订（链接 / 措辞）：PR + CHANGELOG Docs。  
- 改变 P0/P1/P2 硬序或 L0–L5 关门语义：新开 RFC，本页标注修订历史。  
