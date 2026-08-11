# M2 线程模型初稿（`run_concurrent`）

**中文** · 对齐 [`../scheme/milestones.md`](../scheme/milestones.md) M2 DoD。

## 谁驱动事件循环

| 角色 | 线程 | 职责 |
|------|------|------|
| **Pump** | 调用 `Instance.callRunConcurrent` 的线程（仪器 / 应用工作线程） | `pollster::block_on(store.run_concurrent(...))` 泵 CM async 任务 |
| **Host concurrent** | 与 Pump **同一逻辑任务**（在 `run_concurrent` 内被 poll） | `func_wrap_concurrent` 闭包；`FutureReader` 创建 / complete |
| **UI / 主线程** | ART main | **禁止** compile / instantiate / `block_on` 重路径 |

## 约束

1. **单 Store 单泵**：同一 `Store` 不得并发进入两次 `run_concurrent`。
2. **Store 访问**：仅在 `accessor.with(|access| …)` 内碰 `Store`；**不可**跨 `.await` 持有 `Store`/`StoreContextMut`。
3. **延后 complete**：若 oneshot 在另一线程 `send`，必须保证 wake 能回到 Pump；同线程在 `block_on` 里等待自己 `send` 会死锁。当前 M2 smoke 在 concurrent 闭包内同步 `send(42)`。
4. **JNI**：从非附着线程回呼 Kotlin 须 `AttachCurrentThread`（见 `native/src/jvm.rs`）。

## P3 stream 读端（扩展）

| 角色 | 职责 |
|------|------|
| **Host producer** | `StreamReader::new(store, Vec<u8>)`（或其它 `StreamProducer`）；在调用 guest 前创建 |
| **Guest** | canon `stream.read`（本仓 `fixtures/p3/stream_read`） |
| **Pump** | 当前 smoke 用 `call_async` + `pollster::block_on`（与 M2 `run_concurrent` 不同路径，仍须单 Store） |

约束：stream 未 `close` / 未读尽时勿丢弃 `Store`；写端翻转与 stream+future 完成模式另切片。见 [`../scheme/wasi-p3-surface.md`](../scheme/wasi-p3-surface.md) P3-PRIM-3。

## 与轨 A 文档关系

更广的 Dawn / Surface 契约见 [`threading-android.md`](threading-android.md)。M2 仅覆盖 L1 async 泵；接 L2 后不得在 Gpu 线程上嵌套第二个 Store 泵。
