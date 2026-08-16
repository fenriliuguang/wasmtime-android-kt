# 路线图：主推 `wasi:webgpu` 提案

**中文** | （暂无 EN）

> 配套 [`long-term-plan.md`](long-term-plan.md) **P0** · **形状 RFC：** [`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md)（2026-08-16 **Accepted**）。  
> 提案仓库：[WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu)（撰写时 **Phase 2**）。  
> 钉版：`wasi:webgpu@0.3.0-rc.2`（tag `v0.3.0-rc.2`）。  
> **现行切片：S 系列。** W0–W2 史实保留；**W3 / W3+ host-fixed 过渡面冻结，不再扩。**  
> 轨 A 现为展示 Demo，不是本仓 ABI 上游。

## 1. 为什么是 P0

1. **触发点**：标准 / 提案 `wasi:webgpu` 大量方法是 WIT `async func`；轨 A 只能 sync-compat；本仓 M2 已证明官方 Wasmtime 路径可行。  
2. **范围清晰**：提案明确 **不做** windowing；显示面交给 `wasi-gfx` 等——与本仓「不重造 wasi-gfx / 多 window」（NG-9）一致。  
3. **Android 叙事**：真机 Dawn（轨 A L2）+ 薄 L1 是可演示、可回归的宿主路径。  
4. **提案推进**：实现反馈、WIT 钉版、CTS 子集、与上游/Wasmtime 宿主对齐，比空泛「多挂 WASI package」更符合本仓使命。

## 2. 目标与非目标

### 2.1 目标

| ID | 目标 |
|----|------|
| WG-1 | 钉一份提案 WIT 坐标（版本 / commit / RC 标签），与 Guest 工具链一致 |
| WG-2 | 经本仓 L1 **真 CM async** 注册提案中的关键 `async func`（禁止 sync-compat 冒充） |
| WG-3 | GPU Host 继续走 **轨 A `host-api` / `host-webgpu` 当库**（Dawn / Cpu）；本仓拥有 linker / resource / **规范编组** |
| WG-4 | Android 仪器主链：`gpu.request-adapter` / `gpu-adapter.request-device` **WIT 形状**（`option` / `result` / `own`）→ 可观测成功 |
| WG-5 | 向提案 / 生态提供可引用的差距与线程契约（本仓 `docs/mapping`） |
| WG-6 | （中期）渲染或 compute 切片；明确与 `wasi-gfx` 边界（上屏：遗留 experimental surface **或** 提案 `gpu-canvas-context`） |

### 2.2 非目标（本路线图）

- 宣称 **合规 wasi:webgpu 产品** 或通过完整 WebGPU CTS（未另开合规 RFC 前）  
- 在本仓实现 **第二套** GPU Host（NG-7）；允许把轨 A Host **当库**  
- 把 **wasi-gfx / 多 window** 升为与 webgpu 同级短期目标（NG-9）  
- 静默替换轨 A cube **默认** runtime（NG-1）  
- 以桌面 `wasi-webgpu-wasmtime`+wgpu **替换** Android Dawn 主路径  
- 再开 **host-fixed + 过渡 u32** 的 W3+ 功能 PR（NG-12）

## 3. 与现状的差距（起点）

| 现状（M3–M4 归档） | 提案方向 |
|--------------------|----------|
| `experimental:webgpu-cm@0.8.0` 扁平 import 名 | 提案 package / interface 分层 WIT |
| 子集：adapter/device/queue/surface/clear/present | 完整设备 / 资源 / 编解码面远大于子集 |
| 部分路径仍可能受 L2 sync-compat 影响 | 目标：Guest 所见为原生 async；L2 内部可逐步去锁 |
| 上屏用专用 smoke Guest | 长期对齐提案 Guest；上屏边界或需 wasi-gfx / 平台 surface 胶水 |
| 差距史实 | [`../mapping/gap-m4-vs-cube.md`](../mapping/gap-m4-vs-cube.md)（experimental 坐标） |

**W0 交付：** [`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md) — 函数级对照表（experimental 名 ↔ 提案 WIT 名 ↔ async/sync ↔ L2 有无）；提案钉 `wasi:webgpu@0.3.0-rc.2`。

## 4. 切片堆叠（建议序）

活状态（Todo / In Progress / Done）见 GitHub Project [wasmtime-android-kt progress](https://github.com/users/fenriliuguang/projects/1)，**不要**在本页枚举每一刀。本节只定切片**定义与硬序**。

```text
W0  钉版与差距表（文档）
    提案 WIT 版本 · 与 Wasmtime / wit-bindgen 对齐说明 · gap 表

W1  链接与 resource 边界（已交付）
    提案 instance 双注册过渡扁平 `request-adapter` → 同一 L2 sync u32
    见 [`w1-dual-register.md`](w1-dual-register.md)；终态 `[method]` / resource 表属 W3

W2  真 async 主链（硬闸门；adapter + device 已交付）
    request-adapter / adapter-request-device（提案名过渡扁平）`func_wrap_concurrent` + 仪器 `callRunConcurrent`
    禁止 Latch 冒充；非合规宣称；`[method]` 终态名属 W3

W3  队列与缓冲关键面（**过渡史诗已收口，2026-08-16 冻结扩面**）
    下列 `[method]` 已挂名，但 Guest 仍见 **u32 / void**，descriptor/`list`/`option` 由 host 固定——**史实，不是合格形状**。
    完整清单保留于本页 Git 历史与 gap 表；**禁止**再开同类 host-fixed 刀。
    替换走下方 **S 系列**（先形状、再语义）。

W4  呈现路径策略（选型已立；文档）
    遗留 Demo 上屏：experimental surface（选项 A）
    产品形状：提案 `gpu-canvas-context`（须编组后再切；≠ 立刻 wasi-gfx）
    选项 B：wasi-gfx 最小胶水（须单独 RFC，默认不升 P0；= DG-6 / NG-9）
    选项 C：headless compute-only 演示（后期可选）
    见 [`w4-present-strategy.md`](w4-present-strategy.md)

W5  提案反馈与可选 CTS 子集
    文档化 Android/Dawn 特有问题；可选上游 issue；CTS 子集不挡 S2
```

### 4.1 现行硬序：S 系列（规范形状）

权威定义与 DoD：[`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md) §9。摘要：

```text
S0  本 RFC（文档）——结束双轨并行；冻结 host-fixed 扩面

S1  编组脊柱 + `[method]gpu-device.queue`
    (borrow<gpu-device>) -> own<gpu-queue>   禁止再返回过渡 u32

S2  `[method]gpu.request-adapter`
    async → option<own<gpu-adapter>>         真 concurrent；禁止 Latch

S3  `[method]gpu-adapter.request-device`
    async → result<own<gpu-device>, …>

S4  第一个规范 record 入参（如 create-buffer + gpu-buffer-descriptor）

S5  第一个规范 list（如 queue.submit 的 list<borrow<command-buffer>>）

S6+ 按 WIT 替换其余已冻结过渡方法；一 method 一 PR
```

**硬闸门：** S2 若不能真 async ⇒ 停止扩大 option/result 表面，先修 L1 泵（同 W2 / M2）。  
**禁止：** 再开 W3+ host-fixed u32 功能 PR。

## 5. 依赖关系

```text
Wasmtime 追踪（版本含 CM async / 必要时 stream）
    → WASI 0.3 原语（尤其 future；stream 若提案缓冲需要）
        → S0 RFC → S1 编组脊柱 → S2 async option → S3 result → S4 record → S5 list
            → 轨 A Host 仅当后端库（不等扁平面）
```

上游生态对照（**非**运行时依赖）：

| 资产 | 用法 |
|------|------|
| `wasi-webgpu` WIT | 钉版来源；Guest 形状以它为准 |
| `wasi-webgpu-wasmtime` | API 形状 / 宿主经验对照；**不**作 Android 默认 `.so` |
| 轨 A `host-api` / `host-webgpu` | **后端库**（Dawn / Cpu） |
| 轨 A `guest/cube-cm` | **Demo / 遗留** Guest；规范路径换提案坐标 Guest |

## 6. 沟通口径

| 场合 | 说法 |
|------|------|
| 谈本仓使命 | Android JVM 上推进 **官方形状的 wasi:webgpu（提案）** + WASI 0.3，底座为官方 Wasmtime |
| 谈轨 A | **展示用简单 Demo**（experimental cube / sync-compat）；不是本仓上游 |
| 谈「已实现某方法」 | 必须能回答：Guest 看到 WIT 类型，还是过渡 u32 |
| 谈合规 | 未宣布前 **不是** 合规 wasi:webgpu 实现 |
| 谈 wasi-gfx | 显示 / window 另册；本仓不把它当近端 P0 |

## 7. 修订

- S 切片增删、WIT 钉版变更：更新 RFC / 本页对应节 + `changelog/unreleased/` 碎片 +（若有）gap 表。不要为「下一刀」去改根 README 或 `vcs-workflow` 清单；活状态改 Project 卡片。  
- 将 wasi-gfx 升为与本页同级：长期计划修订 RFC。  
- W4 呈现：遗留 experimental surface vs 规范 `gpu-canvas-context` 分层，见 [`w4-present-strategy.md`](w4-present-strategy.md)。  
- 2026-08-16：W3 过渡收口；现行硬序改为 S 系列。  
