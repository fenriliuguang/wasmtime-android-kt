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

## WASI webgpu `[method]gpu.request-adapter`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案 instance 注册 `gpu` resource + sync `get-gpu`；`[method]gpu.request-adapter`：`func_wrap_concurrent` + oneshot yield → 同一 L2 u32 |
| **Guest** | `get-gpu` → `[method]gpu.request-adapter`（borrow self；`fixtures/w1/webgpu_method_request_adapter`） |
| **仪器** | 复用 `attachRequestAdapter`；扁平 `request-adapter` 仍注册 |
| **Pump** | 同 M2：`callRunConcurrent` / `run_concurrent` |

禁止 Latch / 假 future 冒充。非 `option<gpu-adapter>` / options record。

## WASI webgpu `[method]gpu-adapter.request-device`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案 instance 注册 `gpu-adapter` resource + sync `get-adapter`；`[method]gpu-adapter.request-device`：`func_wrap_concurrent` + oneshot yield → L2 `request-adapter` 再 `adapter-request-device`（同 u32） |
| **Guest** | `get-adapter` → `[method]gpu-adapter.request-device`（borrow self；`fixtures/w1/webgpu_method_request_device`） |
| **仪器** | 复用 `attachRequestDevice`；扁平 `adapter-request-device` 仍注册 |
| **Pump** | 同 M2：`callRunConcurrent` / `run_concurrent` |

禁止 Latch / 假 future 冒充。非 `result<gpu-device, request-device-error>` / descriptor。

## WASI webgpu `[method]gpu-device.queue`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | 提案 instance 注册 `gpu-device` resource + sync `get-device`；`[method]gpu-device.queue`：`func_wrap` sync → L2 `request-adapter` 再 `adapter-request-device` 再 `device-get-queue`（同 u32） |
| **Guest** | `get-device` → `[method]gpu-device.queue`（borrow self；`fixtures/w1/webgpu_method_device_queue`） |
| **仪器** | 复用 `attachDeviceGetQueue`；扁平 `device-get-queue` 仍注册 |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方。禁止在泵线程 `AttachCurrentThread` |

非 `gpu-queue` resource / getter 终态类型。

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

## WASI webgpu `[method]gpu-queue.submit`（W3）

| 角色 | 职责 |
|------|------|
| **Host** | `gpu-queue` + `get-queue`；`[method]gpu-queue.submit`：`func_wrap` sync void → L2 queue + encoder/finish + submit1（单 buffer u32） |
| **Guest** | `get-queue` → submit(stub 19)；返回 19（`fixtures/w1/webgpu_method_queue_submit`） |
| **仪器** | 复用 `attachQueueSubmit1` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非提案 `list<borrow<gpu-command-buffer>>`。

## WASI webgpu `[method]gpu-device.create-buffer`（W3+）

| 角色 | 职责 |
|------|------|
| **Host** | 复用 `gpu-device` / `get-device`；`[method]gpu-device.create-buffer`：`func_wrap` sync → L2 adapter/device 再 host 固定 `device-create-buffer`（4 字节 COPY_DST\|VERTEX） |
| **Guest** | `get-device` → create-buffer（`fixtures/w1/webgpu_method_create_buffer`） |
| **仪器** | `attachCreateBuffer` |
| **Pump** | Wasmtime 走 8MiB pthread；L2 JNI 回跳调用方 |

非 Guest `gpu-buffer-descriptor` / `gpu-buffer` resource。

## 与轨 A 文档关系

更广的 Dawn / Surface 契约见 [`threading-android.md`](threading-android.md)。M2 仅覆盖 L1 async 泵；接 L2 后不得在 Gpu 线程上嵌套第二个 Store 泵。
