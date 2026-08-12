# W1 刀切：`wasi:webgpu` 双注册（已交付）

**中文** | （暂无 EN）

> 路线图切片：**W1**（[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) §4）。  
> 差距表：[`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md)。  
> **状态：已交付**（`feat/webgpu-w1-request-adapter`）。下一刀 → **W2**（真 async `request-adapter` / `request-device`）。

## 1. 目的

在 **不撤** experimental 扁平路径的前提下，把提案 **`wasi:webgpu`** 的 package / interface 名挂上 Linker，使至少一条既有 L2 能力能以**提案坐标**被 Guest import。

| 保留 | 新增（W1） |
|------|------------|
| `experimental:webgpu-cm/host@0.8.0#request-adapter`（及现有扁平面） | `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter`（过渡扁平）→ 同一 L2 / u32 路径 |

W1 **不是**合规面、**不是**真 async、**不是**完整 resource 表。

## 2. 钉版（复述 W0；W1 重钉）

| 字段 | 值 |
|------|-----|
| 提案 package | **`wasi:webgpu@0.3.0-rc.2`** |
| tag / commit | **`v0.3.0-rc.2`** → `6a776bada0b66d3dbf9da304a49ff2947ce4e1f8` |
| 来源 | [WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) · `wit/webgpu.wit`（见 gap 表） |

## 3. 交付形态：过渡扁平 `request-adapter`

| 路径 | instance | func |
|------|----------|------|
| experimental（不变） | `experimental:webgpu-cm/host@0.8.0` | `request-adapter` |
| W1 提案名（过渡） | `wasi:webgpu/webgpu@0.3.0-rc.2` | `request-adapter`（**非** `[method]gpu.request-adapter`） |

两条路径共享同一 host 闭包 → `exp_request_adapter` / L2 sync u32。  
终态 resource 方法名与真 async 属 **W2/W3**。

### 3.2 异步边界（硬约束，仍有效）

| 允许 | 禁止 |
|------|------|
| W1 继续 **`func_wrap` sync + u32**（与今日 L2 一致） | 用 Latch / 假 future **冒充**提案 `async func` |
| 文档与测试写明「提案名 + sync-compat 语义」 | 仪器绿灯文案写成「真 CM async」 |

**真 async（`func_wrap_concurrent` / future 完成）属 W2 硬闸门**。

## 4. 落地清单

| 项 | 路径 |
|----|------|
| Linker 双注册 | `native/src/cm.rs` |
| Guest | `fixtures/w1/webgpu_request_adapter.{wat,wasm}` |
| Native smoke | `native/tests/wasi_webgpu_request_adapter.rs`（stub u32=7） |
| 仪器孪生 | `WasiWebGpuRequestAdapterInstrumentedTest` |
| 资产拷贝 | `smoke-app` `copyW1Fixtures` → `androidTest/assets/w1` |

## 5. 明确不在 W1（历史约束）

- present / native window / **wasi-gfx**（W4）  
- 完整 WIT **resource** 表、`[method]gpu.*` 终态名（W3）  
- WebGPU **CTS** 或合规宣称（NG-5）  
- `request-device` 真 async、假 async（**W2**）  
- 静默删除 experimental 扁平面（过渡期双注册）

## 6. 修订

- W1 已交付：本页标「已交付」；下一刀指向 W2。  
- 改 instance / 过渡名：更新本页 + gap §5 + CHANGELOG Docs。
