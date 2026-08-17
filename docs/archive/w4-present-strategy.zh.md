# W4 刀切：呈现路径策略选型

**中文** | （暂无 EN）

> 路线图切片：**W4**（[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) §4）。  
> 差距表：[`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md) 第 4 / 5 / 6 / 12 / 13 行（surface / present；提案无 present）。  
> 非目标：[`non-goals.md`](non-goals.md) **NG-9**、**DG-6**；轨 A 主验收 **NG-1**。  
> **状态：选型已立（文档；2026-08-14）。** **2026-08-16 分层：** experimental surface = **遗留 / Demo 上屏**；产品形状走提案 `gpu-canvas-context`（须编组后再切）。  
> 配套 RFC：[`rfc-wasi-webgpu-canonical-shape.md`](rfc-wasi-webgpu-canonical-shape.md)。  
> 本页是 **策略选型**，**不是**实现 PR，**不是**合规宣称，**不是**切换轨 A cube 默认 runtime。

## 1. 目的

把路线图 W4 的呈现路径写成 **遗留 Demo 上屏 / 规范 canvas / 延期 wasi-gfx**，使规范形状切片（S 系列）不必等待上屏，也不把 wasi-gfx 升成近端 P0。

W4 **不**实现新 Host、**不**撤 experimental surface（Demo 仍可用）、**不**宣称 `wasi:webgpu` 合规。

## 2. 选项分层

| 选项 | 内容 | 近端处置 |
|------|------|----------|
| **A**（遗留） | 继续 experimental surface：`create-surface-from-native-window` / `surface-*` / `surface-present` | **Demo / M4 smoke 上屏**；**不是**产品 WIT 形状 |
| **规范 canvas**（产品方向） | 提案 `gpu-canvas-context`（`configure` / `get-current-texture` / `unconfigure`） | **须编组后再切**；提案仍无 present |
| **B**（延期） | 引入 wasi-gfx 最小胶水 | **须单独 RFC**（= DG-6）；默认不升 P0（NG-9） |
| **C**（后期可选） | headless compute-only 演示 | **后期可选 demo**；**不**替换轨 A cube 默认 runtime（NG-1） |

## 3. 分层：遗留 A vs 规范 canvas

**Demo / 已有 M4 smoke 继续用 A**（experimental surface）。这是遗留上屏，不是 `wasi:webgpu` 合格形状。

**产品路径**对准提案 `gpu-canvas-context`（无 present）。在 S 系列把 record / resource 编组立住之前，**不**把 canvas 当第一刀。

理由：

1. 提案 `wasi:webgpu@0.3.0-rc.2` **没有 present**；`gpu-canvas-context` 止于 `get-current-texture`；显示面交给 wasi-gfx。  
2. **NG-9**：不把 wasi-gfx / 多 window 升为与 `wasi:webgpu` 同级的近端 P0。  
3. **M4** Dawn render smoke 已经走 experimental surface；**S 系列**不得被上屏挡住。  
4. 选项 B（wasi-gfx）= **DG-6**，**须单独 RFC**。  
5. 选项 C 是后期可选 demo，**不**替换轨 A cube 默认 runtime（NG-1）。

长期 L3（规范形状主链）见 [`long-term-plan.md`](long-term-plan.md) 与 RFC；本页不把 experimental surface 写成产品终态。

## 4. S 系列仍可继续做的

S1–S5 按 RFC 先做 **queue resource / async option / result / record / list**，**不必**带 present：

| 可继续 | 说明 |
|--------|------|
| 规范 `gpu-device.queue` 等 | Guest 见 `own` / `borrow`，与 experimental surface **解耦** |
| 渲染 pass / clear 子集 | 可在提案坐标推进 **不 present** 的编码与提交 |

上屏：Demo 仍走 experimental `surface-*`；产品 canvas 另切片。**禁止**再开 W3+ host-fixed 刀。

## 5. 明确不在本刀

- **实现** wasi-gfx / 多 window（选项 B；NG-9 / DG-6）  
- **撤掉** experimental surface 并假装已对齐提案「无 present」面  
- 用选项 C **替换** 轨 A cube 默认 runtime（NG-1）  
- 宣称 **合规** `wasi:webgpu` 或通过完整 WebGPU CTS（NG-5）  
- 把 experimental surface 写成产品终态（产品方向是 `gpu-canvas-context`）

## 6. 对照锚点

| 锚点 | 用法 |
|------|------|
| [`non-goals.md`](non-goals.md) NG-9 | wasi-gfx / 多 window 不升近端 P0 |
| [`non-goals.md`](non-goals.md) DG-6 | 选项 B = 延期 RFC |
| [`non-goals.md`](non-goals.md) NG-1 | 不静默替换轨 A Demo 默认 runtime |
| gap 表 #4 / #5 / #6 / #12 / #13 | 平台 surface；提案无 `surface-present` |
| M4 `render_smoke` | 已用 experimental surface 的 Dawn 上屏史实 |
| RFC | 产品形状 ≠ experimental 扁平面 |

## 7. 修订

- 改变遗留 A / 规范 canvas / B/C 的近端默认：新开 RFC，更新本页 + roadmap W4 + changelog 碎片。  
- 选项 C 若落地为独立 demo PR：链到本页，**不得**改写「Demo 仍可用 A」除非 RFC 明确替换。  
- 2026-08-16：A 降为遗留上屏；产品方向改为 `gpu-canvas-context`。  
