# W1 刀切：`wasi:webgpu` 双注册（规划）

**中文** | （暂无 EN）

> 路线图切片：**W1**（[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) §4）。  
> 差距表：[`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md)。  
> **本文仅规划**；实现走后续短命 `feat/webgpu-w1-…` PR，勿与 clocks / stream 抢同一 `cm.rs` 无协调。

## 1. 目的

在 **不撤** experimental 扁平路径的前提下，把提案 **`wasi:webgpu`** 的 package / interface 名挂上 Linker，使至少一条既有 L2 能力能以**提案坐标**被 Guest import。

| 保留 | 新增（W1） |
|------|------------|
| `experimental:webgpu-cm/host@0.8.0#request-adapter`（及现有扁平面） | 提案 instance 下至少 **一条** 等价注册或别名 → 同一 L2 / u32 路径 |

W1 **不是**合规面、**不是**真 async、**不是**完整 resource 表。

## 2. 钉版（复述 W0）

| 字段 | 值 |
|------|-----|
| 提案 package | **`wasi:webgpu@0.3.0-rc.2`** |
| 来源 | [WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) · `wit/webgpu.wit`（见 gap 表） |
| 实现 PR 义务 | **须重钉**上游 SHA 或 tag，并同步 gap / CHANGELOG；本文不锁死 commit |

## 3. 推荐首刀：`request-adapter`

差距表 #1：experimental `request-adapter` ↔ 提案 **`gpu.request-adapter`**（提案为 `async func`；本仓 L2 今为 sync u32）。

### 3.1 Linker 字符串（建议）

现状（已实现）：

```text
instance  experimental:webgpu-cm/host@0.8.0
func      request-adapter   → L2 sync → u32 adapter rep
```

W1 建议 **双注册同一 host 闭包**（或薄别名），优先试：

```text
instance  wasi:webgpu/webgpu@0.3.0-rc.2
func      [method]gpu.request-adapter
```

说明：

- Instance 形态对齐本仓其它 WASI 注册（如 `wasi:random/random@0.3.0`）：`{package}/{interface}@{version}`。  
- 提案 WIT 入口为 `interface webgpu` + **resource `gpu` 方法**；Component 链接名通常带 `[method]gpu.` 前缀。  
- **若**首刀不做 resource 表 / 无 `gpu` 句柄：允许 **过渡扁平**——仍用 instance `wasi:webgpu/webgpu@0.3.0-rc.2`，func 名暂用 `request-adapter`（与 experimental 同名），在 PR 说明与 gap 表标注「非最终 `[method]` 形」；W2/W3 再收敛到真 resource 方法名。  
- **禁止**只改文档字符串、Guest 仍链 experimental 却宣称「已对齐提案」。

### 3.2 异步边界（硬约束）

| 允许 | 禁止 |
|------|------|
| W1 继续 **`func_wrap` sync + u32**（与今日 L2 一致） | 用 Latch / 假 future **冒充**提案 `async func` |
| 文档与测试写明「提案名 + sync-compat 语义」 | 仪器绿灯文案写成「真 CM async」 |

**真 async（`func_wrap_concurrent` / future 完成）属 W2 硬闸门**，不在本刀。

## 4. 最小代码 DoD（后续 `feat/…` PR）

实现 PR 合并前须同时满足：

1. **双路径可链**：experimental 既有 Guest / 仪器不回归；提案 instance 下至少一条 import 可 link + 调用。  
2. **同一 L2**：两条路径命中同一 Dawn/Cpu `requestAdapter`（或明确共享闭包）；返回仍为可观测 u32（或与今日仪器同构的成功判据）。  
3. **首方法**：优先 `request-adapter` ↔ 提案 `gpu.request-adapter` 命名（§3.1）；若改选其它 gap 行须在 PR 说明理由。  
4. **Guest**：见 §5；仪器：见 §6。  
5. **钉版**：重钉提案 SHA/tag；更新 gap §6 + CHANGELOG。  
6. **不扩大范围**：见 §7。

## 5. Guest fixture 策略

| 选项 | 做法 | 何时选 |
|------|------|--------|
| **A. 新 fixture（推荐）** | 如 `fixtures/w1/webgpu_request_adapter`：只 import 提案 instance 名，调用一条 | 与 m3 experimental Guest 解耦；CI/仪器边界清晰 |
| **B. 双名 Guest** | 同一模块同时 import experimental + 提案名，各调一次 | 想单测「别名等价」；体积与 WIT 工具链更吵 |

默认推 **A**；若工具链暂不能发提案名，可手写 core wasm import 模块名对齐 §3.1，并在 PR 注明「手写过渡，非 wit-bindgen 终态」。

## 6. 仪器策略

| 选项 | 做法 |
|------|------|
| **扩展** `RequestAdapterInstrumentedTest` | 同测内再跑提案名 Guest；断言与现网同一成功语义 |
| **孪生**（如 `WasiWebGpuRequestAdapterInstrumentedTest`） | 专跑 W1 fixture；失败信息不污染 experimental 主测 |

二选一即可；优先 **扩展**（少类、同门禁）。桌面/native 侧可加对称 `#[test]`（命名与 `wasi_random_u64` 同类），不挡「仅仪器」若 CI 已覆盖。

## 7. 明确不在 W1

- present / native window / **wasi-gfx**（W4）  
- 完整 WIT **resource 表**、descriptor / list 编组（W3+）  
- WebGPU **CTS** 或合规宣称（NG-5）  
- `request-device` 真 async、假 async（**W2**）  
- 静默删除 experimental 扁平面（过渡期双注册）

## 8. 预期改动文件（实现 PR，非本文）

| 区域 | 路径（预期） |
|------|----------------|
| Linker 注册 | `native/src/cm.rs`（热点；与其它 feat 协调） |
| 回调 / 桥 | `ExperimentalHostCallbacks` · `ExperimentalWebGpuBridge`（或并列薄包装） |
| 轨 A 跟随 | `abi-cm` / Track A 若导出名或 ABI 需对齐——**本仓 PR 可先双注册，轨 A 跟随另 PR** |
| Fixture | `fixtures/w1/…` 或扩展既有 m3 |
| 测试 | `RequestAdapterInstrumentedTest`（或孪生）· 可选 native `#[test]` |
| 文档同车 | gap 钉版行 · roadmap 状态 · CHANGELOG |

## 9. 风险

| 风险 | 缓解 |
|------|------|
| **package / instance 字符串与 Guest 工具链不一致** | 实现前用一份最小 Guest 试 link；字符串写进测试常量一处 |
| **`[method]gpu.*` vs 过渡扁平名** | §3.1 允许扁平过渡；勿假装已是终态 resource |
| **`cm.rs` 热点** vs clocks / stream PR | 短命分支；合并窗口错开；冲突时 docs/W1 让路已合代码或 rebase |
| **轨 A / abi-cm 未跟** | W1 可只在本仓 Linker 双挂；对外叙事写清「L1 提案名 → 仍 L2 sync」 |
| **误把 sync 写成 async 完成** | DoD §3.2；审 PR 时查 `func_wrap` vs `func_wrap_concurrent` |

## 10. 修订

- 改首选方法、instance 字符串或 DoD：更新本页 + gap §5 + CHANGELOG Docs。  
- W1 代码合入后：本页标「已交付」并链到实现 PR；下一刀指向 W2。
