# 差距表：experimental webgpu-cm ↔ wasi:webgpu 提案

**中文** | （暂无 EN）

> 长期计划 W0（[`../scheme/roadmap-wasi-webgpu.md`](../scheme/roadmap-wasi-webgpu.md)）。  
> 本仓现状：`experimental:webgpu-cm/host@0.8.0` 扁平 sync import（M3–M4）。  
> 提案钉版（核查日 2026-08-11）：[WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) **`wasi:webgpu@0.3.0-rc.2`**（Phase 2；`wit/webgpu.wit`）。  
> **不**宣称合规；上屏 / window 属 wasi-gfx，见 NG-9。

## 1. 坐标对照

| 项 | 本仓（轨 B 现状） | 提案 |
|----|-------------------|------|
| Package | `experimental:webgpu-cm` | `wasi:webgpu` |
| 版本 | `@0.8.0` | `@0.3.0-rc.2`（RC；会漂） |
| Instance / 入口 | 扁平 instance `…/host@0.8.0` | `interface webgpu` + **resource** 方法 |
| Resource 模型 | 全 **u32 rep** 扁平参数 | WIT `resource gpu` / `gpu-adapter` / … |
| 异步 | 全部 `func_wrap` **sync**（L2 内 sync-compat） | 关键路径为 **`async func`**（如 `request-adapter`） |
| 呈现 | `surface-*` + `surface-present`（experimental） | **无** present；`gpu-canvas-context` 止于 `get-current-texture`；显示面 → wasi-gfx |

## 2. 本仓已实现扁平面 → 提案映射

| # | experimental 扁平名（`cm.rs`） | 提案大致对应 | async? | 形状差距 | 优先级 |
|---|-------------------------------|--------------|--------|----------|--------|
| 1 | `request-adapter` | `gpu.request-adapter` | 提案 **async**；本仓 sync | 缺 `option<…options>`；返回 `option<gpu-adapter>` vs `u32` | **W2** |
| 2 | `adapter-request-device` | `gpu-adapter.request-device` | 提案 **async**；本仓 sync | 缺 descriptor / `result<_, request-device-error>` | **W2** |
| 3 | `device-get-queue` | `gpu-device.queue`（getter） | sync | 名与 resource 方法形态 | W3 |
| 4 | `create-surface-from-native-window` | **提案无**（平台 / wasi-gfx） | — | 轨 A Dawn 胶水；非 wasi:webgpu 范围 | W4 策略 |
| 5 | `surface-configure` | `gpu-canvas-context.configure` | sync | 需完整 `gpu-canvas-configuration` record | W4 |
| 6 | `surface-get-current-texture-view` | `get-current-texture` → 再取 view | sync | 提案返回 `gpu-texture`，非直接 view；多一步 | W3/W4 |
| 7 | `device-create-command-encoder` | `gpu-device.create-command-encoder` | sync | 缺 `option<descriptor>` | W3 |
| 8 | `command-encoder-begin-render-pass-clear` | `begin-render-pass` + clear 附件 | sync | 本仓把 clear 颜色塞进专用扁平 API；提案用完整 descriptor | W3 |
| 9 | `render-pass-end` | `gpu-render-pass-encoder.end` | sync | resource 方法名 | W3 |
| 10 | `command-encoder-finish` | `gpu-command-encoder.finish` | sync | 缺 descriptor option | W3 |
| 11 | `queue-submit1` | `gpu-queue.submit` | sync | 提案 `list<borrow<…>>`；本仓单 buffer u32 | W3 |
| 12 | `surface-present` | **提案无** | — | wasi-gfx / 平台 present | W4 |
| 13 | `surface-unconfigure` | `gpu-canvas-context.unconfigure` | sync | 对齐较易 | W4 |

注册落点：`native/src/cm.rs` · `ExperimentalHostCallbacks` · `ExperimentalWebGpuBridge`。  
仪器：`RequestAdapterInstrumentedTest`（#1）· `DawnRenderSmokeInstrumentedTest`（#1–13 子集）。

## 3. 提案有、本仓未覆盖（抽样，非全表）

以下均在 `wasi:webgpu@0.3.0-rc.2` 中存在，**不**在 M4 扁平子集内（完整 WIT ~1k 行）：

| 区域 | 示例 | 备注 |
|------|------|------|
| Adapter 元数据 | `features` / `limits` / `info` | W3+ |
| Device 生命周期 | `destroy`、lost、错误作用域 | W3+ |
| Buffer / Texture / Sampler | 创建、map（**async** map 相关） | 真 async 依赖 L1；禁 sync-compat 冒充 |
| Pipeline / Bind group | render/compute pipeline | cube 主缺口之一 |
| Shader module | WGSL 模块 | |
| Queue 写入 | `write-buffer` / `write-texture` / `on-submitted-work-done`（async） | |
| 查询 / 时间戳 | query-set 等 | 低优 |

与轨 A cube 差距史实仍见 [`gap-m4-vs-cube.md`](gap-m4-vs-cube.md)（experimental 坐标内）。

## 4. 横切差距（比「多个函数」更硬）

| 横切 | 现状 | 目标（长期） |
|------|------|----------------|
| WIT resource + `[method]` 名 | 扁平函数 + u32 | 提案 resource 表；rep 仍可 u32 对齐 L2 |
| Descriptor / list / string 编组 | 几乎无 | 有限集编解码（tech-stack §2.1） |
| 真 CM async 接 GPU | M2 仅假 `get`；webgpu 全 sync | W2：`request-adapter` / `request-device` 走 concurrent |
| Package 字符串 | experimental | 钉提案坐标；可双注册过渡期 |
| 测试 Guest | `fixtures/m3` · `m4/render_smoke` | 提案 WIT 生成或手写标准名 Guest |
| 合规 / CTS | 无 | 另 RFC（NG-5） |

## 5. 建议下一刀（与路线图对齐）

1. **W1**：双注册或别名——提案 interface 名注册至少一条到现有 L2（仍可 u32）；更新 `abi-cm` 跟随策略。  
2. **W2（硬闸门）**：`request-adapter`（及 `request-device`）改为 **真 async** host import；仪器断言非 Latch 冒充。  
3. **W3**：按本表高频方法扩 resource 面；每片独立 DoD。  
4. **W4**：present / native window 书面选 A（继续 experimental surface）/ B（wasi-gfx RFC）/ C（headless）。

## 6. 钉版与修订

| 字段 | 值 |
|------|-----|
| 提案 commit / tag（文档期） | 以 GitHub `main` 上 `0.3.0-rc.2` package 声明为准；实现切片 PR 须重钉 SHA/tag |
| Wasmtime | 见 [`../scheme/wasmtime-tracking.md`](../scheme/wasmtime-tracking.md)（当前 47.x） |
| 修订 | 函数行增删 → 本页 + CHANGELOG；改变提案钉版 → roadmap W0 节同步 |  
