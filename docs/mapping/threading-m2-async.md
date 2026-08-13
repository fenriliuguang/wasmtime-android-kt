# M2 线程模型初稿（`run_concurrent`）

**中文** · 对齐 [`../scheme/milestones.md`](../scheme/milestones.md) M2 DoD。

## 谁驱动事件循环

| 角色 | 线程 | 职责 |
|------|------|------|
| **Pump** | `Instance.callRunConcurrent` 内部 **8MiB** 专用线程 `wasmtime-cm-pump`（调用方 `join` 等待） | `pollster::block_on(store.run_concurrent(...))` 泵 CM async 任务 |
| **Host concurrent** | 与 Pump **同一逻辑任务**（在 `run_concurrent` 内被 poll） | `func_wrap_concurrent` 闭包；`FutureReader` 创建 / complete |
| **UI / 主线程** | ART main | **禁止** compile / instantiate / `block_on` 重路径 |

## 约束

1. **单 Store 单泵**：同一 `Store` 不得并发进入两次 `run_concurrent`。
2. **Store 访问**：仅在 `accessor.with(|access| …)` 内碰 `Store`；**不可**跨 `.await` 持有 `Store`/`StoreContextMut`。
3. **延后 complete**：若 oneshot 在另一线程 `send`，必须保证 wake 能回到 Pump；同线程在 `block_on` 里等待自己 `send` 会死锁。当前 M2 smoke 在 concurrent 闭包内同步 `send(42)`。
4. **JNI**：从非附着线程回呼 Kotlin 须 `AttachCurrentThread`（见 `native/src/jvm.rs`）。  
5. **泵栈**：`callRunConcurrent` 不在调用方线程上 `block_on`；ART 仪器线程约 1MiB，W3 多一跳 sync L2 会 `StackOverflowError`（Vivo 实测）。已合入的 `device-get-queue` 与后续 W3 同步切片共用此泵。

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
| **Pump** | 同读端：`call_async` + `pollster::block_on`；须单 Store |

约束：stream / future 未完成时勿丢弃 `Store`；stdio 等 package 应复用同一 consumer 模式。见 [`../scheme/wasi-p3-surface.md`](../scheme/wasi-p3-surface.md) P3-PRIM-3/4/5。

## WASI cli stdout（复用写端 consumer）

| 角色 | 职责 |
|------|------|
| **Host** | `wasi:cli/stdout@0.3.0#write-via-stream`：与根 `take` 共用 `CollectConsumer` / `pipe`；过渡返回 `future<u32>` 字节数 |
| **Guest** | 同写端流程，import 包名不同（`fixtures/wasi/cli_stdout`；载荷 `OUT\n`） |
| **Pump** | 同写端：`call_async` + `pollster::block_on`（仪器侧复用 `callStreamWrite`） |

## WASI cli stdin（复用读端 producer）

| 角色 | 职责 |
|------|------|
| **Host producer** | `wasi:cli/stdin@0.3.0#read-via-stream`：`StreamReader::new(store, b"IN\n")`；过渡 `func() -> stream<u8>` |
| **Guest** | import → canon `stream.read` → 返回 nbytes（`fixtures/wasi/cli_stdin`） |
| **Pump** | 同写端：`call_async` / 仪器 `callStreamWrite`（`run` 导出） |

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
| **Pump** | 必须走 8MiB `wasmtime-cm-pump`；直接在仪器线程 `nativeCallRunConcurrent` 会在 ~1MiB 栈溢出 |

## 与轨 A 文档关系

更广的 Dawn / Surface 契约见 [`threading-android.md`](threading-android.md)。M2 仅覆盖 L1 async 泵；接 L2 后不得在 Gpu 线程上嵌套第二个 Store 泵。
