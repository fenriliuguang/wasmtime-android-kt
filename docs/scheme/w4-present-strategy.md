# W4 刀切：呈现路径策略选型

**中文** | （暂无 EN）

> 路线图切片：**W4**（[`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) §4）。  
> 差距表：[`../mapping/gap-experimental-vs-wasi-webgpu.md`](../mapping/gap-experimental-vs-wasi-webgpu.md) 第 4 / 5 / 6 / 12 / 13 行（surface / present；提案无 present）。  
> 非目标：[`non-goals.md`](non-goals.md) **NG-9**、**DG-6**；轨 A 主验收 **NG-1**。  
> **状态：选型已立（文档；2026-08-14）。**  
> 本页是 **策略选型**，**不是**实现 PR，**不是**合规宣称，**不是**切换轨 A cube 主验收。

## 1. 目的

把路线图 W4 的三条呈现路径写成**近端默认 / 延期 / 后期可选**，使 W3 队列与编码器切片不必等待上屏策略，也不把 wasi-gfx 升成近端 P0。

W4 **不**实现新 Host、**不**撤 experimental surface、**不**宣称 `wasi:webgpu` 合规。

## 2. 三选项（不发明第四条）

| 选项 | 内容 | 近端处置 |
|------|------|----------|
| **A**（选用） | 继续轨 A experimental surface：`create-surface-from-native-window` / `surface-configure` / `surface-get-current-texture-view` / `surface-present` / `surface-unconfigure` | **近端默认**：Android-first **过渡**上屏路径 |
| **B**（延期） | 引入 wasi-gfx 最小胶水 | **须单独 RFC**（= DG-6）；默认不升 P0（NG-9） |
| **C**（后期可选） | headless compute-only 演示，降低上屏耦合 | **后期可选 demo**；**不**替换 A 作为 Android-first 呈现过渡，**不**替换轨 A cube 主验收（NG-1） |

## 3. 近端默认：选项 A

**近端默认选 A**：保留 Track A experimental surface 作为过渡 present 路径。

理由：

1. 提案 `wasi:webgpu@0.3.0-rc.2` **没有 present**；`gpu-canvas-context` 止于 `get-current-texture`；显示面交给 wasi-gfx。  
2. **NG-9**：不把 wasi-gfx / 多 window 升为与 `wasi:webgpu` 同级的近端 P0。  
3. **M4** Dawn render smoke（`fixtures/m4/render_smoke` · `DawnRenderSmokeInstrumentedTest`）已经走这条 experimental surface 路径；**W3** 的 queue / encoder 切片**不得**等待 present。  
4. 选项 B（wasi-gfx 最小胶水）= **DG-6**，**须单独 RFC**，默认不是 P0。  
5. 选项 C（headless compute-only）是**后期可选 demo**，用来降低 present 耦合；它**不**替换选项 A 作为 Android-first 呈现过渡，也**不**替换轨 A cube 作为主验收（NG-1）。

长期 L3（提案主链）与 L4（双轨合流准备）仍见 [`long-term-plan.md`](long-term-plan.md)；本页不改写那套战略硬序。

## 4. W3 仍可继续做的

W3 继续按差距表扩 **queue / encoder / finish / submit** 等提案名过渡扁平（及后续 `[method]`），**不必**带 present：

| 可继续 | 说明 |
|--------|------|
| `device-get-queue`、`device-create-command-encoder` | 已交付过渡扁平 sync；终态 `[method]` 仍属后续 W3 |
| `command-encoder-finish`、`queue-submit`（及差距表同类） | 提案名路径可切片；与 experimental surface **解耦** |
| 渲染 pass / clear 子集 | 可在提案坐标推进 **不 present** 的编码与提交 |

上屏仍走 experimental `surface-*` / `surface-present`（选项 A），直到另开 RFC 改变本页选型。

## 5. 明确不在本刀

- **实现** wasi-gfx / 多 window（选项 B；NG-9 / DG-6）  
- **撤掉** experimental surface，或把 Guest 上屏切到提案「无 present」面并假装已对齐  
- 用选项 C **替换** 选项 A，或静默替换轨 A cube 主验收（NG-1）  
- 宣称 **合规** `wasi:webgpu` 或通过完整 WebGPU CTS（NG-5）  
- 本 PR **写代码**（native / fixtures / smoke-app / runtime）；本页只落书面选型

## 6. 对照锚点

| 锚点 | 用法 |
|------|------|
| [`non-goals.md`](non-goals.md) NG-9 | wasi-gfx / 多 window 不升近端 P0 |
| [`non-goals.md`](non-goals.md) DG-6 | 选项 B = 延期 RFC |
| [`non-goals.md`](non-goals.md) NG-1 | 不静默替换轨 A cube |
| gap 表 #4 / #5 / #6 / #12 / #13 | 平台 surface；提案无 `surface-present` |
| M4 `render_smoke` | 已用 experimental surface 的 Dawn 上屏史实 |

## 7. 修订

- 改变近端默认（A → B/C）或把 wasi-gfx 升为与 webgpu 同级：新开 RFC，更新本页状态 + roadmap W4 节 + `changelog/unreleased/` 碎片。  
- 选项 C 若落地为独立 demo PR：链到本页，**不得**改写「近端默认 A」除非 RFC 明确替换。  
