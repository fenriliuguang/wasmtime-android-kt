# Android 线程契约（轨 B）

**中文** | [English](threading-android.en.md)

> 初稿（2026-08-10）。继承轨 A [`threading.md`](../../../wasi-webgpu-jvm-mvp/docs/mapping/threading.md) 精神，针对 **自研 L1 + 官方 CM async** 加严。  
> M2 实现前可修订；修订须同步章程。

## 1. 为什么单独写

轨 A sync-compat 下，host 回调往往在 **调用 export 的同一线程** 内阻塞等待 Dawn。  
轨 B 真 async 后：

- Guest 可能挂起；Host 在 **另一时刻** complete future  
- 仍必须遵守 Dawn / `ANativeWindow` / androidx.webgpu 的线程约束  
- `run_concurrent` / event loop 与「谁持有 Surface」必须画清

## 2. 硬规则（草案）

1. **Dawn GPU 对象与 `processEvents`**：与轨 A 相同——约定在 **单一专用线程**（称 `GpuThread`）上使用。  
2. **Surface / 窗口句柄**：创建、configure、present、destroy 与 GpuThread 策略一致；禁止随意跨线程 `windowFromSurface`。  
3. **CM concurrent event loop**：由 **明确的驱动方** 泵（`GpuThread` 或文档指定的 `RuntimeThread`）；禁止多线程同时 `run_concurrent` 同一 Store。  
4. **Future complete**：若 completion 闭包触碰 L2/Dawn，必须 **post 到 GpuThread** 再碰 GPU；或规定 complete 只允许在 GpuThread 调用。  
5. **JNI 附着**：从 Rust async 运行时回呼 Java 必须正确 `AttachCurrentThread` / 分离策略；禁止泄漏附着。  
6. **禁止**：在 ART 主线程做重 compile/instantiate（可接受短 loadLibrary；长活进后台并文档说明）。

## 3. 建议默认模型（M2–M4）

```text
UI / 主线程     ：只负责 Surface 生命周期回调 → 投递到 GpuThread
GpuThread       ：L2 Dawn + processEvents + present + （可选）pump CM loop
Rust async rt   ：只做 Wasmtime 调度；回呼 Java 经线程安全队列
```

若 M2 证明「CM loop 必须与 GpuThread 合一」更简单，则 **合并**，并写进本页修订。

### M4 钉死（2026-08-11）

- 仪器 `DawnRenderSmokeInstrumentedTest`：CM instantiate / host 回调 / Dawn present **均在同一 GpuThread**（匿名后台线程）。  
- UI 线程只做 Surface 生命周期与 Activity 启停；`windowFromSurface` 在 GpuThread 上调用。  
- 仪器启动 `MainActivity` 前须亮屏 / 去锁屏，并用特权 `am start -W`（`targetContext.startActivity` 在 Android 16 / Vivo 上视为后台启动，到不了 `RESUMED`）。  
- M4 首片仍为 sync-compat（无第二 CM async 泵）；与 M2 `run_concurrent` 路径隔离。

## 4. 与轨 A 的差异

| 点 | 轨 A（sync-compat） | 轨 B（目标） |
|----|---------------------|--------------|
| Host import 返回 | 同步返回结果 | 可返回 future，稍后 complete |
| 等待 Dawn | 回调内 latch + processEvents | 发起非阻塞请求 → GpuThread 完成 → complete future |
| 验收 | cube 帧循环现状 | smoke 先证明模型，再追齐帧循环 |

## 5. 验收暗示

- M2：至少证明 complete 发生在文档声明的线程模型下，无数据竞争冒烟。  
- M4：上屏路径无「错误线程碰 Dawn」类崩（对照轨 A blockers 经验）。  
