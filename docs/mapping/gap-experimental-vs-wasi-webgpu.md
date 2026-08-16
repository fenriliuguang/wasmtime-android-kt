# 差距表：experimental webgpu-cm ↔ wasi:webgpu 提案

**中文** | （暂无 EN）

> 长期计划 W0（[`../scheme/roadmap-wasi-webgpu.md`](../scheme/roadmap-wasi-webgpu.md)）。  
> 本仓现状：`experimental:webgpu-cm/host@0.8.0` 扁平 sync import（M3–M4）。  
> 提案钉版（核查日 2026-08-12）：[WebAssembly/wasi-webgpu](https://github.com/WebAssembly/wasi-webgpu) **`wasi:webgpu@0.3.0-rc.2`**（tag `v0.3.0-rc.2` → commit `6a776bada0b66d3dbf9da304a49ff2947ce4e1f8`；Phase 2；`wit/webgpu.wit`）。  
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
| 1 | `request-adapter` | `gpu.request-adapter` | 提案 **async**；**W2** 扁平名真 async；**W3** `[method]gpu.request-adapter` 真 async（`get-gpu` + resource self） | 扁平名仍注册；`[method]` 仍返回过渡 u32（非 `option<gpu-adapter>`）；缺 options | **W3**（`option` / adapter resource 仍后） |
| 2 | `adapter-request-device` | `gpu-adapter.request-device` | 提案 **async**；**W2** 扁平名真 async；**W3** `[method]gpu-adapter.request-device` 真 async（`get-adapter` + resource self） | 扁平名仍注册；`[method]` 仍返回过渡 u32（非 `result<gpu-device, …>`）；缺 descriptor | **W3**（`result` / device resource 仍后） |
| 3 | `device-get-queue` | `gpu-device.queue`（getter） | 提案 **sync**；**W3 首片**提案名路径过渡扁平 sync；**W3** `[method]gpu-device.queue` sync（`get-device` + resource self） | 扁平名仍注册；`[method]` 仍返回过渡 u32（非 `gpu-queue` resource） | **W3**（`gpu-queue` resource 仍后） |
| 4 | `create-surface-from-native-window` | **提案无**（平台 / wasi-gfx） | — | 轨 A Dawn 胶水；非 wasi:webgpu 范围 | W4 策略 |
| 5 | `surface-configure` | `gpu-canvas-context.configure` | sync | 需完整 `gpu-canvas-configuration` record | W4 |
| 6 | `surface-get-current-texture-view` | `get-current-texture` → 再取 view | sync | 提案返回 `gpu-texture`，非直接 view；多一步 | W3/W4 |
| 7 | `device-create-command-encoder` | `gpu-device.create-command-encoder` | 提案 **sync**；**W3** 提案名路径过渡扁平 sync；**W3** `[method]gpu-device.create-command-encoder` sync（`get-device` + resource self） | 扁平名仍注册；`[method]` 仍返回过渡 u32；缺 `option<descriptor>` | **W3**（descriptor 仍后） |
| 8 | `command-encoder-begin-render-pass-clear` | `begin-render-pass` + clear 附件 | 提案 **sync**；**W3** 提案名路径过渡扁平 sync；**W3** `[method]gpu-command-encoder.begin-render-pass` sync（`get-encoder` + resource self + stub view `23`） | 扁平名仍注册；clear 色仍 host 固定；仪器 Cpu 离屏 TextureView 替换 | **W3**（完整 descriptor 仍后） |
| 9 | `render-pass-end` | `gpu-render-pass-encoder.end` | 提案 **sync**；**W3** 提案名路径过渡扁平 sync；**W3** `[method]gpu-render-pass-encoder.end` sync void（`get-pass` + resource self） | 扁平名仍注册；仪器 Cpu 离屏 TextureView | **W3** |
| 10 | `command-encoder-finish` | `gpu-command-encoder.finish` | 提案 **sync**；**W3** 提案名路径过渡扁平 sync；**W3** `[method]gpu-command-encoder.finish` sync（`get-encoder` + resource self） | 扁平名仍注册；缺 descriptor option | **W3**（descriptor 仍后） |
| 11 | `queue-submit1` | `gpu-queue.submit` | 提案 **sync**；**W3** 提案名路径过渡扁平 sync；**W3** `[method]gpu-queue.submit` sync void（`get-queue` + resource self + 单 buffer u32） | 扁平名仍注册；非提案 `list<borrow<…>>` | **W3**（`list` 仍后） |
| 12 | `surface-present` | **提案无** | — | wasi-gfx / 平台 present | W4 |
| 13 | `surface-unconfigure` | `gpu-canvas-context.unconfigure` | sync | 对齐较易 | W4 |

注册落点：`native/src/cm.rs` · `ExperimentalHostCallbacks` · `ExperimentalWebGpuBridge`。  
仪器：`RequestAdapterInstrumentedTest`（#1 experimental）· `WasiWebGpuRequestAdapterInstrumentedTest`（#1 提案名过渡扁平）· `WasiWebGpuMethodRequestAdapterInstrumentedTest`（#1 `[method]gpu.request-adapter`）· `WasiWebGpuRequestDeviceInstrumentedTest`（#2 提案名过渡扁平）· `WasiWebGpuMethodRequestDeviceInstrumentedTest`（#2 `[method]gpu-adapter.request-device`）· `WasiWebGpuDeviceGetQueueInstrumentedTest`（#3 提案名过渡扁平 sync）· `WasiWebGpuMethodDeviceQueueInstrumentedTest`（#3 `[method]gpu-device.queue`）· `WasiWebGpuCreateCommandEncoderInstrumentedTest`（#7 提案名过渡扁平 sync）· `WasiWebGpuMethodCreateCommandEncoderInstrumentedTest`（#7 `[method]gpu-device.create-command-encoder`）· `WasiWebGpuBeginRenderPassInstrumentedTest`（#8 提案名过渡扁平 sync）· `WasiWebGpuMethodBeginRenderPassInstrumentedTest`（#8 `[method]gpu-command-encoder.begin-render-pass`）· `WasiWebGpuRenderPassEndInstrumentedTest`（#9 提案名过渡扁平 sync）· `WasiWebGpuMethodRenderPassEndInstrumentedTest`（#9 `[method]gpu-render-pass-encoder.end`）· `WasiWebGpuCommandEncoderFinishInstrumentedTest`（#10 提案名过渡扁平 sync）· `WasiWebGpuMethodCommandEncoderFinishInstrumentedTest`（#10 `[method]gpu-command-encoder.finish`）· `WasiWebGpuQueueSubmitInstrumentedTest`（#11 提案名过渡扁平 sync）· `WasiWebGpuMethodQueueSubmitInstrumentedTest`（#11 `[method]gpu-queue.submit`）· `WasiWebGpuMethodCreateBufferInstrumentedTest`（W3+ `[method]gpu-device.create-buffer`）· `WasiWebGpuMethodCreateTextureInstrumentedTest`（W3+ `[method]gpu-device.create-texture`）· `WasiWebGpuMethodCreateSamplerInstrumentedTest`（W3+ `[method]gpu-device.create-sampler`）· `WasiWebGpuMethodCreateShaderModuleInstrumentedTest`（W3+ `[method]gpu-device.create-shader-module`）· `WasiWebGpuMethodWriteBufferInstrumentedTest`（W3+ `[method]gpu-queue.write-buffer`）· `WasiWebGpuMethodTextureCreateViewInstrumentedTest`（W3+ `[method]gpu-texture.create-view`）· `WasiWebGpuMethodCreateBindGroupLayoutInstrumentedTest`（W3+ `[method]gpu-device.create-bind-group-layout`）· `WasiWebGpuMethodCreatePipelineLayoutInstrumentedTest`（W3+ `[method]gpu-device.create-pipeline-layout`）· `WasiWebGpuMethodCreateBindGroupInstrumentedTest`（W3+ `[method]gpu-device.create-bind-group`）· `WasiWebGpuMethodCreateRenderPipelineInstrumentedTest`（W3+ `[method]gpu-device.create-render-pipeline`）· `WasiWebGpuMethodCreateComputePipelineInstrumentedTest`（W3+ `[method]gpu-device.create-compute-pipeline`）· `WasiWebGpuMethodWriteTextureInstrumentedTest`（W3+ `[method]gpu-queue.write-texture`）· `WasiWebGpuMethodBeginComputePassInstrumentedTest`（W3+ `[method]gpu-command-encoder.begin-compute-pass`）· `WasiWebGpuMethodComputePassEndInstrumentedTest`（W3+ `[method]gpu-compute-pass-encoder.end`）· `WasiWebGpuMethodCopyBufferToBufferInstrumentedTest`（W3+ `[method]gpu-command-encoder.copy-buffer-to-buffer`）· `WasiWebGpuMethodComputePassSetPipelineInstrumentedTest`（W3+ `[method]gpu-compute-pass-encoder.set-pipeline`）· `WasiWebGpuMethodComputePassDispatchWorkgroupsInstrumentedTest`（W3+ `[method]gpu-compute-pass-encoder.dispatch-workgroups`）· `WasiWebGpuMethodRenderPassSetPipelineInstrumentedTest`（W3+ `[method]gpu-render-pass-encoder.set-pipeline`）· `WasiWebGpuMethodRenderPassDrawInstrumentedTest`（W3+ `[method]gpu-render-pass-encoder.draw`）· `WasiWebGpuMethodComputePassSetBindGroupInstrumentedTest`（W3+ `[method]gpu-compute-pass-encoder.set-bind-group`）· `WasiWebGpuMethodRenderPassSetBindGroupInstrumentedTest`（W3+ `[method]gpu-render-pass-encoder.set-bind-group`）· `WasiWebGpuMethodRenderPassSetVertexBufferInstrumentedTest`（W3+ `[method]gpu-render-pass-encoder.set-vertex-buffer`）· `DawnRenderSmokeInstrumentedTest`（#1–13 子集）。

## 3. 提案有、本仓未覆盖（抽样，非全表）

以下均在 `wasi:webgpu@0.3.0-rc.2` 中存在，**不**在 M4 扁平子集内（完整 WIT ~1k 行）：

| 区域 | 示例 | 备注 |
|------|------|------|
| Adapter 元数据 | `features` / `limits` / `info` | W3+ |
| Device 生命周期 | `destroy`、lost、错误作用域 | W3+ |
| Buffer / Texture / Sampler | 创建、map（**async** map 相关） | **W3** `[method]gpu-device.create-buffer` / `[method]gpu-device.create-texture` / `[method]gpu-device.create-sampler` / `[method]gpu-texture.create-view` sync 已交付（host 固定 descriptor，仍 u32）；map 仍后 |
| Pipeline / Bind group | render/compute pipeline | **W3** `[method]gpu-device.create-bind-group-layout` / `[method]gpu-device.create-pipeline-layout` / `[method]gpu-device.create-bind-group` / `[method]gpu-device.create-render-pipeline` / `[method]gpu-device.create-compute-pipeline` sync 已交付（host 固定空 entries / stub shader，仍 u32）；descriptor / list 仍后 |
| Shader module | WGSL 模块 | **W3** `[method]gpu-device.create-shader-module` sync 已交付（host 固定 WGSL，仍 u32） |
| Queue 写入 | `write-buffer` / `write-texture` / `on-submitted-work-done`（async） | **W3** `[method]gpu-queue.write-buffer` / `[method]gpu-queue.write-texture` sync 已交付（host 固定 4 字节 / 1×1，单 buffer/texture u32）；async 仍后 |
| Command encoding | begin-compute-pass / copy-buffer / pass set-pipeline / draw / set-bind-group / set-vertex-buffer | **W3** `[method]gpu-command-encoder.begin-compute-pass` / `[method]gpu-compute-pass-encoder.end` / `[method]gpu-command-encoder.copy-buffer-to-buffer` / `[method]gpu-compute-pass-encoder.set-pipeline` / `[method]gpu-compute-pass-encoder.dispatch-workgroups` / `[method]gpu-render-pass-encoder.set-pipeline` / `[method]gpu-render-pass-encoder.draw` / `[method]gpu-compute-pass-encoder.set-bind-group` / `[method]gpu-render-pass-encoder.set-bind-group` / `[method]gpu-render-pass-encoder.set-vertex-buffer` sync 已交付（无 descriptor / host 固定 4 字节 copy / stub pipeline / 1×1×1 dispatch / vertexCount 3 / 空 bind-group index 0 / VERTEX buffer slot 0，仍 u32 / void） |
| 查询 / 时间戳 | query-set 等 | 低优 |

与轨 A cube 差距史实仍见 [`gap-m4-vs-cube.md`](gap-m4-vs-cube.md)（experimental 坐标内）。

## 4. 横切差距（比「多个函数」更硬）

| 横切 | 现状 | 目标（长期） |
|------|------|----------------|
| WIT resource + `[method]` 名 | 扁平函数 + u32；**W3 高频面已挂** `gpu` / `gpu-adapter` / `gpu-device` / `gpu-command-encoder` / `gpu-render-pass-encoder` / `gpu-compute-pass-encoder` / `gpu-queue` 及对应 `[method]`；**W3+** `[method]gpu-device.create-buffer` / `[method]gpu-device.create-texture` / `[method]gpu-device.create-sampler` / `[method]gpu-device.create-shader-module` / `[method]gpu-queue.write-buffer` / `[method]gpu-texture.create-view` / `[method]gpu-device.create-bind-group-layout` / `[method]gpu-device.create-pipeline-layout` / `[method]gpu-device.create-bind-group` / `[method]gpu-device.create-render-pipeline` / `[method]gpu-device.create-compute-pipeline` / `[method]gpu-queue.write-texture` / `[method]gpu-command-encoder.begin-compute-pass` / `[method]gpu-compute-pass-encoder.end` / `[method]gpu-command-encoder.copy-buffer-to-buffer` / `[method]gpu-compute-pass-encoder.set-pipeline` / `[method]gpu-compute-pass-encoder.dispatch-workgroups` / `[method]gpu-render-pass-encoder.set-pipeline` / `[method]gpu-render-pass-encoder.draw` / `[method]gpu-compute-pass-encoder.set-bind-group` / `[method]gpu-render-pass-encoder.set-bind-group` / `[method]gpu-render-pass-encoder.set-vertex-buffer`（返回仍 u32 / void） | 提案 resource 表；rep 仍可 u32 对齐 L2；descriptor / `list` / `option` 仍后 |
| Descriptor / list / string 编组 | 几乎无 | 有限集编解码（tech-stack §2.1） |
| 真 CM async 接 GPU | M2 `get`；**W2** 提案名 `request-adapter` + `adapter-request-device` concurrent；experimental 仍 sync | 更多 GPU async；`[method]` resource |
| Package 字符串 | experimental + W1 双注册 `wasi:webgpu/webgpu@0.3.0-rc.2`（过渡扁平 + **W3** `[method]gpu.request-adapter` / `[method]gpu-adapter.request-device` / `[method]gpu-device.queue` / `[method]gpu-device.create-command-encoder` / **W3+** `[method]gpu-device.create-buffer` / `[method]gpu-device.create-texture` / `[method]gpu-device.create-sampler` / `[method]gpu-device.create-shader-module` / `[method]gpu-queue.write-buffer` / `[method]gpu-texture.create-view` / `[method]gpu-device.create-bind-group-layout` / `[method]gpu-device.create-pipeline-layout` / `[method]gpu-device.create-bind-group` / `[method]gpu-device.create-render-pipeline` / `[method]gpu-device.create-compute-pipeline` / `[method]gpu-queue.write-texture` / `[method]gpu-command-encoder.begin-compute-pass` / `[method]gpu-compute-pass-encoder.end` / `[method]gpu-command-encoder.copy-buffer-to-buffer` / `[method]gpu-compute-pass-encoder.set-pipeline` / `[method]gpu-compute-pass-encoder.dispatch-workgroups` / `[method]gpu-render-pass-encoder.set-pipeline` / `[method]gpu-render-pass-encoder.draw` / `[method]gpu-compute-pass-encoder.set-bind-group` / `[method]gpu-render-pass-encoder.set-bind-group` / `[method]gpu-render-pass-encoder.set-vertex-buffer`） | 收敛到更多 `[method]gpu.*` resource 名 |
| 测试 Guest | `fixtures/m3` · `fixtures/w1` · `m4/render_smoke` | 提案 WIT 生成或手写更多 `[method]` Guest |
| 合规 / CTS | 无 | 另 RFC（NG-5） |

## 5. 下一刀（活状态在看板）

切片**定义**仍见 §2 各行与 [`../scheme/roadmap-wasi-webgpu.md`](../scheme/roadmap-wasi-webgpu.md)。  
**正在做 / 下一刀** 只维护在 GitHub Project：[wasmtime-android-kt progress](https://github.com/users/fenriliuguang/projects/1)（筛 `Slice` = W3 / W4，`Status` = Todo 或 In Progress）。功能 PR **不要**在本页追加编号清单。

史实（硬闸门，不随每刀改写）：W0 差距表已交付；W1 双注册见 [`../scheme/w1-dual-register.md`](../scheme/w1-dual-register.md)；W2 adapter/device 真 async 已过闸。W3 起按本表高频方法扩面并收敛 `[method]` 名；W4 选型已书面落地，见 [`../scheme/w4-present-strategy.md`](../scheme/w4-present-strategy.md)。

## 6. 钉版与修订

| 字段 | 值 |
|------|-----|
| 提案 commit / tag（W1 重钉） | tag **`v0.3.0-rc.2`** → commit **`6a776bada0b66d3dbf9da304a49ff2947ce4e1f8`**（package `wasi:webgpu@0.3.0-rc.2`） |
| Wasmtime | 见 [`../scheme/wasmtime-tracking.md`](../scheme/wasmtime-tracking.md)（当前 47.x） |
| 修订 | 函数行增删 → 本页 + CHANGELOG；改变提案钉版 → roadmap W0 节同步 |  
