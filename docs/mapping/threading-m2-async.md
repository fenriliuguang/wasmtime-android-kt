# M2 线程模型初稿（`run_concurrent`）

**中文** · 对齐 [`../scheme/milestones.md`](../scheme/milestones.md) M2 DoD。

## 谁驱动事件循环

| 角色 | 线程 | 职责 |
|------|------|------|
| **Pump** | `nativeCallRunConcurrent` 内 **8MiB pthread** `wasmtime-cm-pump`（调用方 `join` 并代跑 L2 JNI） | `pollster::block_on(store.run_concurrent(...))` 泵 CM async 任务 |
| **Host concurrent** | 与 Pump **同一逻辑任务**（在 `run_concurrent` 内被 poll） | `func_wrap_concurrent` 闭包；`FutureReader` 创建 / complete |
| **UI / 主线程** | ART main | **禁止** compile / instantiate / `block_on` 重路径 |

## 约束

1. **单 Store 单泵**：同一 `Store` 不得并发进入两次 `run_concurrent`。
2. **Store 访问**：仅在 `accessor.with(|access| …)` 内碰 `Store`；**不可**跨 `.await` 持有 `Store`/`StoreContextMut`。
3. **延后 complete**：若 oneshot 在另一线程 `send`，必须保证 wake 能回到 Pump；同线程在 `block_on` 里等待自己 `send` 会死锁。当前 M2 smoke 在 concurrent 闭包内同步 `send(42)`。
4. **JNI**：从非附着线程回呼 Kotlin 须 `AttachCurrentThread`（见 `native/src/jvm.rs`）。  
5. **泵栈**：Wasmtime `block_on` 在 8MiB pthread 上跑；L2 Kotlin 回跳必须回到调用方 JNI 线程。ART 仪器线程约 1MiB，W3 多一跳 sync L2 会 `StackOverflowError`。Java `Thread(stackSize)` 被忽略；在自定义栈 pthread 上 `AttachCurrentThread` 会 abort（`FindStackTop` vs `GetStackEnd`，Vivo / Android 16）。

## P3 stream 读端（扩展）

| 角色 | 职责 |
|------|------|
| **Host producer** | `StreamReader::new(store, Vec<u8>)`（或其它 `StreamProducer`）；在调用 guest 前创建 |
| **Guest** | canon `stream.read`（本仓 `fixtures/p3/stream_read`） |
| **Pump** | 当前 smoke 用 `call_async` + `pollster::block_on`（与 M2 `run_concurrent` 不同路径，仍须单 Store） |

## P3 stream 写端 / 写方向翻转（扩展）

| 角色 | 职责 |
|------|------|
| **Host consumer** | 根 import `take(stream<u8>) -> future<u32>`：`StreamReader::pipe` + `StreamConsumer`；`FutureReader` 在 consumer drop 时完成字节数 |
| **Guest** | `stream.new` → `take` → canon `stream.write` → `drop-writable` → `future.read`（`fixtures/p3/stream_write`） |
| **Pump** | 同 M2：`nativeCallStreamWrite` 走 8MiB pthread `run_concurrent` / `call_concurrent`（ART 仪器线程 ~1MiB 会栈溢出崩进程） |

约束：stream / future 未完成时勿丢弃 `Store`；stdio 等 package 应复用同一 consumer 模式。空探针 `poll_consume` 只返回 `Pending`，禁止 `wake_by_ref`（会在 guest `stream.write` 栈上同步重入）。见 [`../scheme/wasi-p3-surface.md`](../scheme/wasi-p3-surface.md) P3-PRIM-3/4/5。

## WASI cli stdout（复用写端 consumer）

| 角色 | 职责 |
|------|------|
| **Host** | `wasi:cli/stdout@0.3.0#write-via-stream`：与根 `take` 共用 `CollectConsumer` / `pipe`；过渡返回 `future<u32>` 字节数 |
| **Guest** | 同写端流程，import 包名不同（`fixtures/wasi/cli_stdout`；载荷 `OUT\n`） |
| **Pump** | 同写端：仪器 `callStreamWrite`（8MiB pthread `run_concurrent`） |

## WASI cli stdin（复用读端 producer）

| 角色 | 职责 |
|------|------|
| **Host producer** | `wasi:cli/stdin@0.3.0#read-via-stream`：`StreamReader::new(store, b"IN\n")`；过渡 `func() -> stream<u8>` |
| **Guest** | import → canon `stream.read` → 返回 nbytes（`fixtures/wasi/cli_stdin`） |
| **Pump** | 同写端：`callStreamWrite`（8MiB pthread） |

## WASI cli command-shaped async `run`

| 角色 | 职责 |
|------|------|
| **Guest** | 根导出 `run: async func() -> u32`（过渡 0=ok）；import 已有 `wasi:cli/stdout@0.3.0#write-via-stream`，写 `CMD\n`（`fixtures/wasi/cli_command`） |
| **Host consumer** | 与 stdout 相同：`CollectConsumer` / `pipe`；生产路径不改 `cm.rs` |
| **Pump** | `callRunConcurrent` / `run_concurrent`（与 monotonic `wait-for` 同泵）；stdout consumer 不变 |

## WASI webgpu request-adapter / request-device（W2）

| 角色 | 职责 |
|------|------|
| **Host concurrent** | `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter` / `#adapter-request-device`：`func_wrap_concurrent`；oneshot + helper-thread yield → L2 |
| **Guest** | async import + async export `run`（`fixtures/w1/webgpu_request_adapter` · `webgpu_request_device`） |
| **Pump** | 同 M2：`callRunConcurrent` / `run_concurrent` |
| **Experimental** | `experimental:webgpu-cm/host@0.8.0` 对应扁平面仍 `func_wrap` sync |

禁止 Latch / 假 future 冒充。

## WASI webgpu device-get-queue（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案名过渡扁平 `device-get-queue`：`func_wrap` sync → 同一 L2 u32 |
| **Guest** | async adapter → async device → sync get-queue（`fixtures/w1/webgpu_device_get_queue`） |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

## WASI webgpu command-encoder-finish（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案名过渡扁平 `command-encoder-finish`：`func_wrap` sync → 同一 L2 u32 |
| **Guest** | async adapter → async device → sync encoder + finish（`fixtures/w1/webgpu_command_encoder_finish`） |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

## WASI webgpu queue-submit1（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案名过渡扁平 `queue-submit1`：`func_wrap` sync void → 同一 L2（单 command-buffer u32） |
| **Guest** | async adapter → async device → sync queue + encoder + finish + submit（`fixtures/w1/webgpu_queue_submit`） |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

## WASI webgpu begin-render-pass-clear（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案名过渡扁平 `command-encoder-begin-render-pass-clear`：`func_wrap` sync → 同一 L2 u32 |
| **Guest** | async adapter → async device → sync encoder + begin-clear（stub view `23`；`fixtures/w1/webgpu_begin_render_pass`） |
| **仪器** | `attachBeginRenderPassClear` 在拿到 device 后建 Cpu 1×1 离屏 TextureView，替换 Guest stub view；**不**走 experimental surface / present |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

## WASI webgpu render-pass-end（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案名过渡扁平 `render-pass-end`：`func_wrap` sync void → 同一 L2 u32 |
| **Guest** | async adapter → async device → sync encoder + begin-clear（stub view `23`）+ end（`fixtures/w1/webgpu_render_pass_end`） |
| **仪器** | `attachRenderPassEnd` 同 begin-clear 替换 Cpu 离屏 TextureView 后再 `renderPassEnd`；**不**走 experimental surface / present |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

## WASI webgpu `[method]gpu.request-adapter`（S2）

| 角色 | 职责 |
|------|------|
| **Host** | 提案 instance：`gpu` + `gpu-adapter` resource；sync `get-gpu`（测试构造器）；`[method]gpu.request-adapter`：`func_wrap_concurrent` + oneshot yield → L2 `request-adapter`，返回 `option<own<gpu-adapter>>`（表内 `GpuAdapter.rep` = L2 u32） |
| **Guest** | `get-gpu` → `[method]gpu.request-adapter`(self, none) → drop own adapter；`run` 返回 harness `1` |
| **仪器** | 复用 `attachRequestAdapter`；扁平 `request-adapter` 仍注册 |
| **Pump** | 同 M2：`callRunConcurrent` / `run_concurrent` |

禁止 Latch / 假 future 冒充。形状与钉版 WIT `request-adapter: async func(options: option<gpu-request-adapter-options>) -> option<gpu-adapter>` 同构。非合规宣称。

## WASI webgpu `[method]gpu-adapter.request-device`（S3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案 instance：`gpu-adapter` + `gpu-device` + `record-option-gpu-size64` resource；sync `get-adapter`（测试构造器）；`[method]gpu-adapter.request-device`：`func_wrap_concurrent` + oneshot yield → L2 `adapter-request-device`，返回 `result<own<gpu-device>, request-device-error>`（表内 `GpuDevice.rep` = L2 u32） |
| **Guest** | `get-adapter` → `[method]gpu-adapter.request-device`(self, none) → drop own device on ok；`run` 返回 harness `1` |
| **仪器** | 复用 `attachRequestDevice`；扁平 `adapter-request-device` 仍注册 |
| **Pump** | 同 M2：`callRunConcurrent` / `run_concurrent` |

禁止 Latch / 假 future 冒充。形状与钉版 WIT `request-device: async func(descriptor: option<gpu-device-descriptor>) -> result<gpu-device, request-device-error>` 同构。非合规宣称。Guest 本刀传 descriptor=none（S4 才填真实 record）。

## WASI webgpu `[method]gpu-device.queue`（S1）

| 角色 | 职责 |
|------|------|
| **Host** | 提案 instance：`gpu-device` + `gpu-queue` resource；sync `get-device`（测试构造器）；`[method]gpu-device.queue`：`func_wrap` → L2 adapter/device/queue，**返回 `own<gpu-queue>`**（表内 `GpuQueue.rep` = L2 u32） |
| **Guest** | `get-device` → `[method]gpu-device.queue`（borrow self → own queue）→ `[resource-drop]gpu-queue`；`run` 返回 harness `1` |
| **仪器** | 复用 `attachDeviceGetQueue`；扁平 `device-get-queue` 仍注册 |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

形状与钉版 WIT `queue: func() -> gpu-queue` 同构。非合规宣称。

## WASI webgpu `[method]gpu-device.create-command-encoder`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-command-encoder`：`func_wrap` sync → L2 `request-adapter` 再 `adapter-request-device` 再 `device-create-command-encoder`（同 u32） |
| **Guest** | `get-device` → `[method]gpu-device.create-command-encoder`（borrow self；`fixtures/w1/webgpu_method_create_command_encoder`） |
| **仪器** | 复用 `attachCreateCommandEncoder`；扁平 `device-create-command-encoder` 仍注册 |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

非 `option<command-encoder-descriptor>`。

## WASI webgpu `[method]gpu-command-encoder.begin-render-pass`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | `gpu-command-encoder` + `get-encoder`；`[method]gpu-command-encoder.begin-render-pass`：`func_wrap` sync → L2 adapter/device/encoder/begin-clear（view u32） |
| **Guest** | `get-encoder` → begin（stub view `23`；`fixtures/w1/webgpu_method_begin_render_pass`） |
| **仪器** | 复用 `attachBeginRenderPassClear`（Cpu 离屏 TextureView 替换 stub） |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非完整 `gpu-render-pass-descriptor`。

## WASI webgpu `[method]gpu-render-pass-encoder.end`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | `gpu-render-pass-encoder` + `get-pass`；`[method]gpu-render-pass-encoder.end`：`func_wrap` sync void → L2 begin-clear(stub 23) 再 end |
| **Guest** | `get-pass` → end；返回 stub 29（`fixtures/w1/webgpu_method_render_pass_end`） |
| **仪器** | 复用 `attachRenderPassEnd` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

## WASI webgpu `[method]gpu-command-encoder.finish`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-command-encoder` / `get-encoder`；`[method]gpu-command-encoder.finish`：`func_wrap` sync → L2 adapter/device/encoder/finish |
| **Guest** | `get-encoder` → finish（`fixtures/w1/webgpu_method_command_encoder_finish`） |
| **仪器** | 复用 `attachCommandEncoderFinish` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 `option<command-buffer-descriptor>`。

## WASI webgpu `[method]gpu-queue.submit`（S5）

| 角色 | 职责 |
|------|------|
| **Host** | `gpu-queue` + `gpu-command-buffer` + `get-queue` + `get-command-buffer`；`[method]gpu-queue.submit`：`func_wrap` sync `(borrow<gpu-queue>, list<borrow<gpu-command-buffer>>) -> ()` → table.get 每个 list 元素；L2 仍 host-fixed encoder/finish + submit1 |
| **Guest** | `get-queue` + `get-command-buffer` → submit(单元素 list) → drop owns；`run` 返回 1（`fixtures/w1/webgpu_method_queue_submit`） |
| **仪器** | 复用 `attachQueueSubmit1` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

形状与钉版 WIT `submit: func(command-buffers: list<borrow<gpu-command-buffer>>)` 同构。`get-command-buffer` 仅测试构造器。非合规宣称。

## WASI webgpu `[method]gpu-device.create-buffer`（S4）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-buffer`：`func_wrap` sync `(borrow, gpu-buffer-descriptor) -> own<gpu-buffer>` → L2 adapter/device 再按 Guest size/usage 建 buffer |
| **Guest** | `get-device` → create-buffer（size=4，COPY_DST\|VERTEX，mapped/label=none）→ drop own；`run` 返回 1（`fixtures/w1/webgpu_method_create_buffer`） |
| **仪器** | `attachCreateBuffer`（转发 Guest size/usage） |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

形状与钉版 WIT `create-buffer: func(descriptor: gpu-buffer-descriptor) -> gpu-buffer` 同构。mapped-at-creation / label 本刀传 none。非合规宣称。

## WASI webgpu `[method]gpu-device.create-texture`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-texture`：`func_wrap` sync → L2 adapter/device + host-fixed 1×1 RGBA8 RENDER_ATTACHMENT |
| **Guest** | `get-device` → create-texture（`fixtures/w1/webgpu_method_create_texture`） |
| **仪器** | `attachCreateTexture` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-texture-descriptor` / `gpu-texture` resource。

## WASI webgpu `[method]gpu-device.create-sampler`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-sampler`：`func_wrap` sync → L2 adapter/device + host-fixed default sampler |
| **Guest** | `get-device` → create-sampler（`fixtures/w1/webgpu_method_create_sampler`） |
| **仪器** | `attachCreateSampler` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `option<gpu-sampler-descriptor>` / `gpu-sampler` resource。

## WASI webgpu `[method]gpu-device.create-shader-module`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-shader-module`：`func_wrap` sync → L2 adapter/device + host-fixed stub WGSL |
| **Guest** | `get-device` → create-shader-module（`fixtures/w1/webgpu_method_create_shader_module`） |
| **仪器** | `attachCreateShaderModule` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-shader-module-descriptor` / `gpu-shader-module` resource。

## WASI webgpu `[method]gpu-queue.write-buffer`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-queue` / `get-queue`；`[method]gpu-queue.write-buffer`：`func_wrap` sync void → L2 adapter/device/queue + host-fixed create-buffer + 4-byte write |
| **Guest** | `get-queue` → write-buffer(stub 31)；返回 31（`fixtures/w1/webgpu_method_write_buffer`） |
| **仪器** | `attachWriteBuffer` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非提案 `list<u8>` / `borrow<gpu-buffer>`。

## WASI webgpu `[method]gpu-texture.create-view`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | `gpu-texture` + `get-texture`；`[method]gpu-texture.create-view`：`func_wrap` sync → L2 adapter/device + host-fixed 1×1 texture + create-view |
| **Guest** | `get-texture` → create-view（`fixtures/w1/webgpu_method_texture_create_view`） |
| **仪器** | `attachCreateTextureView` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `option<gpu-texture-view-descriptor>` / `gpu-texture-view` resource。

## WASI webgpu `[method]gpu-device.create-bind-group-layout`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-bind-group-layout`：`func_wrap` sync → L2 adapter/device + host-fixed empty entries |
| **Guest** | `get-device` → create-bind-group-layout（`fixtures/w1/webgpu_method_create_bind_group_layout`） |
| **仪器** | `attachCreateBindGroupLayout` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-bind-group-layout-descriptor` / `gpu-bind-group-layout` resource。

## WASI webgpu `[method]gpu-device.create-pipeline-layout`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-pipeline-layout`：`func_wrap` sync → L2 adapter/device + host-fixed empty bind-group-layouts |
| **Guest** | `get-device` → create-pipeline-layout（`fixtures/w1/webgpu_method_create_pipeline_layout`） |
| **仪器** | `attachCreatePipelineLayout` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `option<gpu-pipeline-layout-descriptor>` / `gpu-pipeline-layout` resource。

## WASI webgpu `[method]gpu-device.create-bind-group`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-bind-group`：`func_wrap` sync → L2 adapter/device + host-fixed empty BGL then empty entries |
| **Guest** | `get-device` → create-bind-group（`fixtures/w1/webgpu_method_create_bind_group`） |
| **仪器** | `attachCreateBindGroup` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-bind-group-descriptor` / `gpu-bind-group` resource。

## WASI webgpu `[method]gpu-device.create-render-pipeline`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-render-pipeline`：`func_wrap` sync → L2 adapter/device + host-fixed stub WGSL + triangle RGBA8 |
| **Guest** | `get-device` → create-render-pipeline（`fixtures/w1/webgpu_method_create_render_pipeline`） |
| **仪器** | `attachCreateRenderPipeline` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-render-pipeline-descriptor` / `gpu-render-pipeline` resource。

## WASI webgpu `[method]gpu-device.create-compute-pipeline`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-compute-pipeline`：`func_wrap` sync → L2 adapter/device + host-fixed stub WGSL + empty pipeline-layout |
| **Guest** | `get-device` → create-compute-pipeline（`fixtures/w1/webgpu_method_create_compute_pipeline`） |
| **仪器** | `attachCreateComputePipeline` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-compute-pipeline-descriptor` / `gpu-compute-pipeline` resource；Cpu 要求显式 layout。

## WASI webgpu `[method]gpu-queue.write-texture`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-queue` / `get-queue`；`[method]gpu-queue.write-texture`：`func_wrap` sync void → L2 adapter/device/queue + host-fixed 1×1 COPY_DST texture + 4-byte write |
| **Guest** | `get-queue` → write-texture(stub 37)；返回 37（`fixtures/w1/webgpu_method_write_texture`） |
| **仪器** | `attachWriteTexture` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非提案 `gpu-texel-copy-texture-info` / `list<u8>`；不复用 RENDER_ATTACHMENT `create-texture`。

## WASI webgpu `[method]gpu-command-encoder.begin-compute-pass`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-command-encoder` / `get-encoder`；`[method]gpu-command-encoder.begin-compute-pass`：`func_wrap` sync → L2 adapter/device/encoder + host-default compute-pass descriptor |
| **Guest** | `get-encoder` → begin-compute-pass；返回 pass rep（`fixtures/w1/webgpu_method_begin_compute_pass`） |
| **仪器** | `attachBeginComputePass` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-compute-pass-descriptor` / `gpu-compute-pass-encoder` resource。

## WASI webgpu `[method]gpu-compute-pass-encoder.end`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 新 `gpu-compute-pass-encoder` / `get-compute-pass`；`[method]gpu-compute-pass-encoder.end`：`func_wrap` sync void → L2 adapter/device/encoder + begin-compute-pass + end（忽略 Guest stub） |
| **Guest** | `get-compute-pass` → end；返回 stub 79（`fixtures/w1/webgpu_method_compute_pass_end`） |
| **仪器** | `attachComputePassEnd` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

不复用 `get-pass`（那是 render-pass）。Cpu `computePassEnd` 会 drop handle。

## WASI webgpu `[method]gpu-command-encoder.copy-buffer-to-buffer`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-command-encoder` / `get-encoder`；`[method]gpu-command-encoder.copy-buffer-to-buffer`：`func_wrap` sync void → L2 adapter/device/encoder + host 建 COPY_SRC/COPY_DST buffer 再 copy 4 字节（忽略 Guest stub src/dst） |
| **Guest** | `get-encoder` → copy(stub 31, 31)；返回 stub 31（`fixtures/w1/webgpu_method_copy_buffer_to_buffer`） |
| **仪器** | `attachCopyBufferToBuffer` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非提案 offsets / size 从 Guest 传入。

## WASI webgpu `[method]gpu-compute-pass-encoder.set-pipeline`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-compute-pass-encoder` / `get-compute-pass`；`[method]gpu-compute-pass-encoder.set-pipeline`：`func_wrap` sync void → L2 adapter/device/encoder + begin-compute-pass + host-fixed stub shader / empty layout compute pipeline（忽略 Guest stub pipeline） |
| **Guest** | `get-compute-pass` → set-pipeline(stub 73)；返回 stub 73（`fixtures/w1/webgpu_method_compute_pass_set_pipeline`） |
| **仪器** | `attachComputePassSetPipeline` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-compute-pipeline` resource。

## WASI webgpu `[method]gpu-compute-pass-encoder.dispatch-workgroups`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-compute-pass-encoder` / `get-compute-pass`；`[method]gpu-compute-pass-encoder.dispatch-workgroups`：`func_wrap` sync void → L2 adapter/device/encoder + begin-compute-pass + host-fixed set-pipeline + empty bind-group 0 + dispatch(1,1,1)（忽略 Guest counts） |
| **Guest** | `get-compute-pass` → dispatch(1,1,1)；返回 stub 79（`fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups`） |
| **仪器** | `attachComputePassDispatchWorkgroups` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

Cpu 要求 pipeline 与 bind-group index 0 均已 set。

## WASI webgpu `[method]gpu-render-pass-encoder.set-pipeline`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-render-pass-encoder` / `get-pass`；`[method]gpu-render-pass-encoder.set-pipeline`：`func_wrap` sync void → L2 adapter/device/encoder + begin-render-pass-clear（Cpu 离屏 view）+ host-fixed triangle pipeline（忽略 Guest stub pipeline） |
| **Guest** | `get-pass` → set-pipeline(stub 71)；返回 stub 71（`fixtures/w1/webgpu_method_render_pass_set_pipeline`） |
| **仪器** | `attachRenderPassSetPipeline` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-render-pipeline` resource。

## WASI webgpu `[method]gpu-render-pass-encoder.draw`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-render-pass-encoder` / `get-pass`；`[method]gpu-render-pass-encoder.draw`：`func_wrap` sync void → L2 adapter/device/encoder + begin-render-pass-clear（Cpu 离屏 view）+ host-fixed triangle set-pipeline + draw(3)（忽略 Guest vertexCount） |
| **Guest** | `get-pass` → draw(3)；返回 stub 29（`fixtures/w1/webgpu_method_render_pass_draw`） |
| **仪器** | `attachRenderPassDraw` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

Cpu 只校验 `vertexCount >= 0`。

## WASI webgpu `[method]gpu-compute-pass-encoder.set-bind-group`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-compute-pass-encoder` / `get-compute-pass`；`[method]gpu-compute-pass-encoder.set-bind-group`：`func_wrap` sync void → L2 adapter/device/encoder + begin-compute-pass + host-fixed empty bind-group index 0（忽略 Guest stub bind-group） |
| **Guest** | `get-compute-pass` → set-bind-group(stub 67)；返回 stub 67（`fixtures/w1/webgpu_method_compute_pass_set_bind_group`） |
| **仪器** | `attachComputePassSetBindGroup` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

Cpu 只接受 bind-group index 0。非提案 `option` / `list` / `result`。

## WASI webgpu `[method]gpu-render-pass-encoder.set-bind-group`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-render-pass-encoder` / `get-pass`；`[method]gpu-render-pass-encoder.set-bind-group`：`func_wrap` sync void → L2 adapter/device/encoder + begin-render-pass-clear（Cpu 离屏 view）+ host-fixed empty bind-group index 0（忽略 Guest stub bind-group） |
| **Guest** | `get-pass` → set-bind-group(stub 67)；返回 stub 67（`fixtures/w1/webgpu_method_render_pass_set_bind_group`） |
| **仪器** | `attachRenderPassSetBindGroup` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

Cpu 只校验 bind-group handle 与 `index >= 0`。非提案 `option` / `list` / `result`。

## WASI webgpu `[method]gpu-render-pass-encoder.set-vertex-buffer`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-render-pass-encoder` / `get-pass`；`[method]gpu-render-pass-encoder.set-vertex-buffer`：`func_wrap` sync void → L2 adapter/device/encoder + begin-render-pass-clear（Cpu 离屏 view）+ host-fixed VERTEX buffer slot 0（忽略 Guest stub buffer） |
| **Guest** | `get-pass` → set-vertex-buffer(stub 31)；返回 stub 31（`fixtures/w1/webgpu_method_render_pass_set_vertex_buffer`） |
| **仪器** | `attachRenderPassSetVertexBuffer` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

Cpu 只校验 buffer handle 与 `slot/offset/size >= 0`。非提案 `option` slot/offset/size。

## WASI webgpu `[method]gpu-buffer.map-async`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 新 `gpu-buffer` / `get-buffer`；`[method]gpu-buffer.map-async`：`func_wrap_concurrent` 真 async void → oneshot yield 后 L2 adapter/device + host-fixed MAP_READ buffer 再 map（忽略 Guest stub buffer） |
| **Guest** | `get-buffer` → map-async；返回 stub 31（`fixtures/w1/webgpu_method_buffer_map_async`） |
| **仪器** | `attachBufferMapAsync` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

禁止 Latch 冒充。非提案 `mode` / `offset` / `result<_, map-async-error>`。

## WASI webgpu `[method]gpu-buffer.unmap`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-buffer` / `get-buffer`；`[method]gpu-buffer.unmap`：`func_wrap` sync void → L2 adapter/device + host-fixed MAP_READ buffer 先 map 再 unmap（忽略 Guest stub buffer） |
| **Guest** | `get-buffer` → unmap；返回 stub 31（`fixtures/w1/webgpu_method_buffer_unmap`） |
| **仪器** | `attachBufferUnmap` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非提案 `result<_, unmap-error>`。

## 与轨 A 文档关系

更广的 Dawn / Surface 契约见 [`threading-android.md`](threading-android.md)。M2 仅覆盖 L1 async 泵；接 L2 后不得在 Gpu 线程上嵌套第二个 Store 泵。
