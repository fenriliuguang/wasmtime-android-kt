# 立方体连续 present 抖动排查表

[English](gfx-hitch-checklist.md) | **中文**

不是切刀队列。真机调查：仓外旋转立方体（Vivo V2458A，Android 16，`arm64-v8a`，Mali-G925-Immortalis MC12，120 Hz）。连续 `wasi-gfx` `on-frame`，不是 GFXV 仪器那 500 ms。不要 vendor demo。不要给上游提 GitHub issue。

下列 **Check** 以本页中文表为准；英文页与本页同步。H12 之后整体流畅；剩下 **约 5 s 小抖** 时 acquire 仍 8.3 ms。H1–H28 不要再叠 DisplayManager 投票。**远因**（运行时接线 / guest / Wasmtime / 合成器）见 §6。不要 vendor demo。不要给上游提 GitHub issue。

相关：[`gap-webgpu-wit-androidx.zh.md`](gap-webgpu-wit-androidx.zh.md) §5、[`threading-android.md`](threading-android.md) §8、[`frame-loop-suggestion.zh.md`](frame-loop-suggestion.zh.md)。Guest 时钟 / dt 在**仓外** examples 仓。Dawn C A/B（去掉 androidx JNI 后仍抖）：[`gfx-hitch-native-dawn.zh.md`](gfx-hitch-native-dawn.zh.md)。

**Check**

| 标记 | 含义 |
|------|------|
| **Open** | 仍可能造成抖动；真机尚未证伪 |
| **Likely** | 每帧都会走这条路径；当前最可疑 |
| **Mitigated** | 本分支已改；画面仍抖，单独不够 |
| **Closed** | 对**这次抖动**已排除（仍可能是崩溃/泄漏事实） |
| **Guest** | 仓外 wasm / demo host；不在本仓树里 |
| **Trace** | 需要 systrace / 逐帧日志；本轮未做 |

## 1. 已关闭或已减轻（列出以免再踩）

| ID | 假设 | Check | 证据 |
|----|------|-------|------|
| C1 | BLAST / HandleTable 泄漏交换链 `GPUTexture` | **Closed**（崩溃） | present 后从不 `tryDrop` → 抽搐升高后 GpuThread SIGSEGV `0x20` ~10 s。回收 + GPU fence；V2458A 上崩溃消失（约数分钟）。 |
| C2 | 在当帧 present / 下一 acquire / 纯 CPU 帧环上 `close()` | **Closed**（UAF） | 立刻 close → `0x20` / `0x1f8`。keep 3 **且** GPU 完成。keep-8 抽干 BLAST 池。 |
| C3 | Host 把 `postGfxVsync` 锁成 ~60 Hz | **Mitigated** | 已去掉 `MIN_GFX_VSYNC_NS`。开局过快消失；仍抖。该 cap 本身会在 120 Hz 上造成隔拍抖动。 |
| C4 | Guest 每拍 `angle += const` | **Mitigated** / **Guest** | 立方体已用 `wasi:clocks/monotonic-clock#now`（rAF 式 dt，50 ms 钳制）。Pin `frame-event` 仍是 `{ nothing: bool }`。 |
| C5 | 吞掉帧中途到达的 vsync | **Mitigated** | Native `GfxOnFrameGate`：`pending \|\| in_frame` 时 `post` 丢拍。仍抖 → 不是唯一原因。测试 `wasi_gfx_frame_loop_vsync_paced` 已更新。 |
| C6 | GFXV 仪器本身 | **Closed** | `CLOSE_AFTER_VSYNC_MS = 500`；从未看到数秒 present。Cpu 回收：`WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest`。 |
| C7 | 每帧 `onSubmittedWorkDone` JNI 全局引用泄漏（androidx.webgpu） | **Mitigated**（崩溃） | `GPUQueue.onSubmittedWorkDone` 每次调用对 **`callback` 和 `executor` 各**泄漏一个 JNI global ref（从不 `DeleteGlobalRef`）。2 refs/帧 → `global reference table overflow (max=51200)`，GpuThread `SIGABRT`，约 3.5 min（25,320 帧）。Dump：25,312 × `ExternalSyntheticLambda4`（1 unique = 共享 executor）+ 25,308 × `queueSubmit$2`（每帧 callback）。改为每 2 帧 fence 一次，速率减半（**实测**：overflow 由 25,320 → ~50,640 帧，3.5 → 7.0 min @120Hz）。根治靠上游 / 自研 wgpu FFI——本 AAR 的 `libwebgpu_c_bundled.so` 只导出 `Java_androidx_webgpu_*`，无 `wgpuQueueOnSubmittedWorkDone`。 |
| C8 | H21 竞态：`processEvents`（持锁）vs `getCurrentTexture`（不持锁）→ Mali SIGSEGV | **Open**（崩溃，偶发） | eventPoller `processEvents` → Dawn `vulkan::driver::QueueSubmit` → `libGLES_mali` 空指针（`signal 11`，fault `0x0`）。第一次 90s 触发；第二次 423s 直接到 C7 overflow 未触发。重新持锁 acquire 会再次阻塞 poller（H21）。 | 复现再排查。 |

## 2. 剩余原因（按此顺序 check）

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| H1 | vsync 之后、`get-current-texture` **等待上一帧 GPU**（`awaitCanvasGpuDone`），present 相对 scanout 偏晚 | **Mitigated** | 已去掉 acquire 前等待。acquire 即时（`last` 0.2–0.6 ms，`SuccessOptimal`）。UAF 环仍在。 | — |
| H2 | `in_frame` 丢拍 + 120 Hz：帧时在 8.3 ms 两侧振荡 → **60/120 抖动** | **Mitigated** | 曾距上次 take 数 2 拍（稳 60 fps）。V2458A mode 3 60 Hz + 60 fps 仍回退 BLAST（H27）。现与 Choreographer **1:1**。 | 120 Hz 面板看画面。 |
| H3 | `clocks.now` 是进程 `Instant`，不是 `Choreographer.frameTimeNanos` | **Mitigated** | Host-only：`in_frame` 时 `now` 是本拍 vsync instant（Choreographer dt 累加到 WASI epoch）。不是 WIT `frame-event` instant。 | 与 H9 一起看画面。 |
| H4 | `queueWriteBuffer` **每次 `ByteBuffer.allocateDirect`** | **Mitigated** | 复用一块 direct buffer；`synchronized(gpuLock)`。 | 真机仍抖则不是 H4 单独。 |
| H5 | `queueWriteBuffer` **不拿 `gpuLock`**，而 `eventPoller` 在该锁下 `processEvents` | **Mitigated** | `writeBuffer` 持 `gpuLock`。 | 若 SIGSEGV 回来，并入 H21。 |
| H6 | `PresentMode.Fifo` + `GPUSurface.present()` **在此 AAR 上不等** GPU/合成器 | **Mitigated** | 120 Hz 1:1 上 Mailbox 把 BLAST 加到 **6** 张，acquire 8/16 ms 混。改回 Fifo + 窗口缓冲上限（H9）。 | 看画面。 |
| H7 | 不在 `awaitCanvasGpuDone` 时，fence 被 **`POLL_MS = 5`** 推迟 | **Mitigated** | acquire 等待路径已去掉。canvas fence 在飞时 poller sleep 1 ms，否则 5 ms。 | — |
| H8 | WG-6 **`queue.submit` 自动 present** 后再 guest `context.present` | **Closed**（双 present） | `presentPendingCanvasFrameLocked` 幂等（pending 已清）。第二次 present 是 no-op。不是每帧两次 present。 | — |
| H9 | SurfaceView / BLAST 额外缓冲 vs Choreographer | **Closed**（app 压不了） | Mailbox：**6** 张。`setBufferCount(3)` = EINVAL。`setBufferCount(4)` rc=0，gfxinfo 仍 **5** 张。 | 与 H27 同为合成器下限。 |
| H10 | GpuThread 上 CM `stream.read` / `callRunConcurrent` 成本 | **Closed**（这次 5 s 抖动） | 会是连续抖。H12 之后 Choreographer / acquire 仍 ~8.3 ms。 | — |
| H11 | 每帧 guest `resource.drop`（texture/view/encoder/cb） | **Closed**（这次 5 s 抖动） | 每帧 JNI，不是 5 s 周期。 | — |
| H12 | wait 开始后再等一拍（`start_gen+1`）把 120 Hz 压成 60 fps | **Mitigated** | 未消费拍仍丢。`last_take_gen` 锁到当前 generation（积压拍一次 present）。真机：Choreographer 8.3 ms 时 acquire `lastDtNs` ~16 ms。 | 直方图应往 `<11ms` 靠。 |
| H13 | Guest dt **50 ms 钳制** | **Closed**（这次抖动） | 只在停顿 ≥50 ms 时钳。用户看到的是频繁小顿，不是 50 ms 跳。 | — |
| H14 | 首帧 dummy **1/60 s** dt（`last_ns == 0`） | **Closed**（仅开局） | 一帧。用户抖动是连续的。 | — |
| H15 | 每帧上传 **列主序 mat4** `write_buffer` | **Mitigated** | 并入 H4（scratch buffer）。 | — |
| H16 | `GpuThread`（`HandlerThread`）上 ART GC / JIT | **Closed**（这次 5 s 抖动） | JNI `jbyteArray` 已复用。H12 之后 acquire **没有** `>20ms`；进程 GC 间隔 **约 87 s**，不是 5 s。 | 只有分配型抖动回来才做 Release minify。 |
| H17 | submit 之后 guest 仍调 `canvasContextPresent` | **Closed** | 同 H8；no-op。 | — |
| H18 | Depth / MSAA / 过大交换链 | **Closed** | 立方体在 `depth24plus`→RGBA8 事实后跳过 depth。交换链是窗口大小。 | — |
| H19 | 两个 Choreographer 回调（`doFrame` 里 `postFrameCallback`） | **Closed** | 标准 vsync 注册；一条回调链。本身不是双拍。 | — |
| H20 | 产品 linker 缺 `monotonic-clock.now` | **Closed** | 已在 `native/src/cm.rs` 注册；立方体 import 了。 | — |
| H21 | `GPUSurface.getCurrentTexture` 持 `gpuLock` 且可能在 Dawn/BLAST **里阻塞** | **Mitigated** | acquire **不持** `gpuLock`，poller 可 `processEvents` + retire。打 acquire ns 与状态（`GfxHitch`）。Timeout/Suboptimal 仍是抖动信号。 | 真机看 `GfxHitch` acquire 警告。 |
| H22 | `onSubmittedWorkDone` executor 是内联 `Runnable::run` | **Closed**（不是第二座钟） | `callbackExecutor = Executor(Runnable::run)`：fence 跑在调用 `processEvents` 的线程（现为 poller）。不是双 vsync。 | 并入 H5。 |
| H23 | UI `Choreographer` → JNI `postGfxVsync` → GpuThread condvar | **Closed**（这次 5 s 抖动） | `FullscreenSurface` 直方图全是 `<11ms`，约 5 s 那一下仍在。 | — |
| H24 | 窗口 / Surface 刷新率 ≠ Choreographer 120 Hz | **Guest** / **Mitigated** | 峰值 `preferredDisplayModeId` + 内容 `setFrameRate` 同 Hz（不再钉 60）。 | `FullscreenSurface` vs Choreographer 直方图。 |
| H25 | `deviceGetQueue` **每次 insert 新** HandleTable `Queue`，且无 `gpuLock` | **Mitigated** / **Guest** | Host intern 一个 queue handle 并持 `gpuLock`。立方体已缓存 `device.queue()`。 | — |
| H26 | `awaitCanvasGpuDone` 泵 `processEvents` 时 `tryDrop` 旧交换链 | **Mitigated** | fence 回调只 `countDown`。`retireGpuDoneCanvasFramesLocked` 只在 poller 的 `processEvents` 之后跑，不在 acquire 等待中。 | 真机若 close() 仍卡，看 H16/H9。 |
| H27 | Android 16 VRR / 显示模式切换 | **Closed**（app 侧） | 强行 SF 120 → 抖 **约 3 s**（已撤回）。`CHANGE_FRAME_RATE_ONLY_IF_SEAMLESS` **仍约 5 s**。弹出时 Choreographer/acquire 仍 8.3 ms。`mAlwaysRespectAppRequest=false`；Vivo `vivo_rms_screen`。 | 系统设置锁 120 Hz / 游戏模式。不要再叠投票。 |
| H28 | 每帧 `textureCreateView` / encoder / bind-group 的 CM+JNI | **Closed**（这次 5 s 抖动） | 同 H10/H11：每帧成本，不是 5 s 回退。 | — |

## 3. 建议杀序

真机已做完。**不要**再叠 DisplayManager / GameState / SurfaceControl 投票。

1. **H12** — 已落地：1:1 出帧，整体流畅。  
2. **H27/H9** — 剩下 **约 5 s** 不是应用漏 vsync。  
3. **§6** — 远因。下一探针是合成器 `timestats` / FrameTimeline，不是再钉 Wasmtime。

## 4. 本分支已改

- 产品 canvas 交换链环 + 异步 `onSubmittedWorkDone`；**下一 acquire 不再等**上一帧 fence；poller 上 retire，GPU 完成且再过 **3** 帧才 close（H1/H26/C2）。  
- `onSubmittedWorkDone` 改为**每 2 帧** batch fence 一次（C7：androidx.webgpu 每次调用泄漏 callback+executor 各一个 global ref）。  
- `getCurrentTexture` 不持 `gpuLock`；acquire ns + 间隔直方图（`GfxHitch`）（H21/H2）。  
- `queueWriteBuffer` 持 `gpuLock` 并复用 direct buffer（H4/H5）。  
- Fifo + configure 前 `ANativeWindow_setBufferCount(4)`（H9；3 = EINVAL，4 仍压不到 gfxinfo 5 张以下）。intern 一个 device queue（H25）。  
- 去掉 60 Hz `postGfxVsync` cap。  
- `native/src/host.rs` 里 `in_frame` vsync 丢拍；**每一拍**都消费；积压拍一次 present，不再额外等 wait 开始后的一拍（H2/H12/H27）。  
- Host-only：`on-frame` 期间 `clocks.now` 是该拍 vsync instant（H3）。  
- Cpu host pending/last canvas 回收；帧寿命仪器。  
- 仓外立方体：`clocks.now` dt（不在本 git 树）。examples host：峰值显示模式 + 同 Hz `setFrameRate` + `setPreferMinimalPostProcessing`（H9/H24/H27）。

## 5. 真机行

| 设备 | C1–C5 之后仍抖 | 备注 |
|------|----------------|------|
| Vivo V2458A（PD2415 / V2458A），Mali-G925，120 Hz | **整体流畅；约 5 s 小抖** | H12：弹出时 acquire 仍 ~8.3 ms。强行 SF 120 → 约 3 s。远因：§6。2026-08-27。 |

## 6. H12 之后的远因（接线 / guest / Wasmtime / 合成器）

**筛选。** 约 5 s 那一下发生时，`FullscreenSurface` Choreographer 和 `GfxHitch` acquire 仍约 8.3 ms（`>20ms=0`）。凡是会 **卡住** `run_concurrent`、GpuThread、MoonBit GC、或 Wasmtime epoch/fuel 中断的路径，都会在 `lastDtNs` 上冒尖。这些对这次抖动是 **Closed**。剩下的只能是：(a) 应用仍在按时 present，SurfaceFlinger **回退积压的 BLAST 图**；或 (b) 不误 app vsync 的 OEM 策略。**D24 修正（2026-08-29）：**纯 androidx.webgpu 原生立方体计时同样干净但**无约 5 s 弹出**，因此 (a)/(b) 单独不构成抖动——它是 Wasmtime/`host-dawn` present 路径特有（重开 D2/D3）。

依据：本树（`native/src/engine.rs`、`cm.rs` 的 `poll_produce` / `nativeCallRunConcurrent`、host-dawn present、仓外 `run.mbt`）；[AOSP frame pacing](https://source.android.com/docs/core/graphics/frame-pacing)（SF 等到同相位才 latch）；[AOSP 游戏循环 / buffer stuffing](https://android.googlesource.com/platform/docs/source.android.com/+/master/en/devices/graphics/arch-gameloops.html)；[Perfetto FrameTimeline](https://perfetto.dev/docs/data-sources/frametimeline)（`BUFFER_STUFFING`、`PREDICTION_ERROR`「周期性自校正」）；[Wasmtime 中断执行](https://docs.wasmtime.dev/examples-interrupting-wasm.html)（epoch/fuel — **本 Engine 未开**）。不要给上游提 GitHub issue；Android 事实只记本仓。

### 6.1 运行时接线

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| D1 | 同步 pin `on-frame` + 阻塞 `poll_produce`（不用 `Poll::Pending`） | **Closed**（这次 5 s 抖动） | `GfxOnFrameProducer` 在 GpuThread 上 condvar 等待：pin 是同步 `func`，WAT 在 stream.read `BLOCKED` 会 trap。这是 **每一拍**，也是 acquire 8.3 ms 的原因，不是 5 s 定时器。 | stackful CM async 是以后的产品事，不是这次弹出。 |
| D2 | UI vsync → JNI `postGfxVsync` → 8MiB `wasmtime-cm-pump` → Dawn present | **Closed**（卡住型） | 泵是 8MiB pthread 上 `pollster::block_on` + `run_concurrent`/`call_concurrent`；L2 JNI 弹回 GpuThread。节拍 1:1。5 s 卡住会进直方图。 | — |
| D3 | Dawn `present()` **没有** presentation timestamp（无 Swappy / `eglPresentationTimeANDROID`） | **Open** / **Likely** | Host 从不设目标 present 时刻。AOSP：把 BufferQueue 塞满再靠 back-pressure 会加延迟；队列满时 SF 可能 **重播** 或 **丢帧**。gfxinfo：SurfaceView **5** 张 BLAST。对得上「回到上一两帧」。 | `dumpsys SurfaceFlinger --timestats` 的 `presentToPresent`；Perfetto `BUFFER_STUFFING`。实验：每 N 帧跳过一次 present 抽干队列（AOSP 游戏循环写法）。不要 Mailbox（池子会到 6）。 |
| D4 | 双 present（`queue.submit` 自动 present + guest `ctx.present`） | **Closed** | H8：第二次 present 是 no-op。 | — |
| D5 | Event poller `POLL_MS` 5 / fence retire keep-3 | **Closed**（这次 5 s 抖动） | 5 ms sleep 不是 5 s 周期。Keep-3 是约 25 ms GPU 寿命，不是合成器回退。 | — |
| D6 | `clocks.now` 接线 vs Choreographer | **Closed**（这次 5 s 抖动） | H3：`in_frame` 时 `now` 是本拍 vsync instant。合成器回退仍会 **显示旧图**，即使 guest dt 正确。 | — |
| D7 | 泵上的 JNI / ART GC | **Closed**（这次 5 s 抖动） | H16：进程 GC 间隔约 87 s；acquire 没有 `>20ms`。 | — |

### 6.2 Guest（仓外 MoonBit 立方体）

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| D8 | MoonBit `async-core` 调度 / 多余协程 | **Closed**（这次 5 s 抖动） | `run` 是 `on_frame()` 然后 `frames.read(1)` 的 `for`。Host 用 CM `run: func() -> u32` + `call_concurrent` 驱动。多余 yield 会推迟 acquire。直方图是干净的。 | — |
| D9 | `frames.read(1)` 先 0 长度就绪再 skip | **Closed**（这次 5 s 抖动） | `chunk.length() == 0` → `continue`（不 present）。那是 **应用漏一帧**（直方图空洞），不是 5 s 回退旧 BLAST。 | 以后 guest 若出现 0 长度 read，打 skip 计数。 |
| D10 | 每帧 `create_view` / encoder / `drop` WIT own | **Closed**（这次 5 s 抖动） | 必须 drop，否则 Wasmtime resource 表涨到 `nativeCallRunConcurrent` SIGSEGV。`color_tex.drop` 在 host 上只动表。成本是每帧。 | guest 复用 encoder/view 是 CPU 优化，不是 5 s 修复。 |
| D11 | `frame_delta_sec` 50 ms 钳 / 首帧 1/60 | **Closed** | H13/H14。用户看到的是回退 1–2 帧，不是 50 ms 转角跳。 | — |
| D12 | Pin `frame-event` 是 `{ nothing: bool }`（WIT 里没有 rAF 时间戳） | **Mitigated** / **Guest** | Guest 改用 `monotonic-clock.now`。WIT 不变就带不了 Choreographer ns。 | acquire 已 1:1 则不是这次抖动。 |
| D13 | Guest 转角 vs 合成器回退 | **Likely**（症状） | SF 重播 BLAST *n−2* 时，立方体 **看起来** 往回走，即使 `angle` 已经加过。用来区分 guest dt 和 D3。 | Perfetto：app present 时间单调 vs SF latch 往回跳。 |

### 6.3 Wasmtime（本 Engine）

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| D14 | Epoch 中断或 fuel（周期性 yield/trap） | **Closed** | `native/src/engine.rs`：只开了 `wasm_component_model` + `wasm_component_model_async`。没有 `epoch_interruption`，没有 fuel。[Wasmtime 中断文档](https://docs.wasmtime.dev/examples-interrupting-wasm.html) 在我们 opt-in 之前不适用。 | 不要在立方体 Store 上开 epoch。 |
| D15 | Cranelift / pooling allocator / 约每 5 s 的 memory grow | **Closed**（这次 5 s 抖动） | 编译/实例化只一次。Guest 是固定 buffer 的紧循环（864 + 64 字节写入）。grow 或编译停顿会卡住 acquire。 | — |
| D16 | CM async `run_concurrent` 内部定时器 | **Closed**（这次 5 s 抖动） | 泵是整段会话一次 `call_concurrent("run")` 的 `pollster::block_on`。这个包装里没有 5 s host 定时器。 | — |
| D17 | Resource 表压缩 / dtor 风暴 | **Closed**（这次 5 s 抖动） | Guest **故意** 每帧 drop 5 个 own。表无限涨是崩溃，不是 5 s 抖动。交换链 dtor 只动表。 | — |
| D18 | 「Wasmtime 慢 / CM 实验性」当成 5 s 周期 | **Closed**（这次抖动） | 正因为 CM async 才有 `on-frame`。CM 慢会是 **连续** 60/120 抖（H12 之前那种）。H12 之后周期形状像合成器。 | 不要给 Wasmtime 提 issue。 |

### 6.4 合成器 / OEM（网上 + 真机）

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| D19 | 5 张 BLAST BufferQueue stuffing；SF 重播再抽干 | **Open** / **Likely** | 同 D3。AOSP：出帧快于显示则丢；慢则重播上一张。5 深队列 + 无 present timestamp。 | `timestats` P90/P99 `presentToPresent`；若确认 stuffing 再试偶尔 skip-present。 |
| D20 | SF「等到同相位」+ UID 被节流到 120 的约数（60/90） | **Open** | [AOSP FPS throttling](https://source.android.com/docs/core/graphics/frame-pacing)：GameManager/RMS 若把 UID 映到 60/90，SF 会等到同相位。App Choreographer 仍可以是 120。 | `dumpsys SurfaceFlinger` 图层 frame-rate override vs Choreographer。**不要**再设 GameState（曾加到约 3 s）。 |
| D21 | VSyncPredictor `PREDICTION_ERROR`，调度器「周期性校正」 | **Open** / **Trace** | [Perfetto](https://perfetto.dev/docs/data-sources/frametimeline)：预测漂移算 jank；孤立误差常常无感——**塞满** 的队列会让一次校正看起来像回退 1–2 帧。 | 对着 5 s 弹出采 Perfetto。 |
| D22 | Vivo RMS / `vivo_rms_screen` / 自适应 60·90·120 | **Open** / **Likely** | 设备有 `vivo_rms_screen`；`mAlwaysRespectAppRequest=false`；强行 SF 120 **提高** 抖动频率。Vivo「智能切换」是已知闪烁类问题。 | 用户设置：锁 120 Hz、关掉智能切换。不要用 runtime 投票。 |
| D23 | Kernel / DisplayModeDirector 空闲降刷新超时 | **Open**（弱） | 立方体每拍 present，图层并不空闲。厂商 SF 里仍常见数秒 idle timer。 | 打开「显示刷新率」叠加层，看弹出时 Hz 是否掉。 |
| D24 | 不含 Wasmtime 的原生 androidx.webgpu 立方体也会同样抖 | **Closed** — 上游单独**不会**复现 | `hosts/native-webgpu` `CubeActivity`（纯 androidx.webgpu 1.0.0-alpha05，无 Wasmtime、无 `host-dawn`）在 V2458A 上：1:1 出帧 @120 Hz，Choreographer ~8.3 ms（0 `>20ms`），acquire ~8.3 ms（0 `>20ms`，`SuccessOptimal`），显示钉在 120 Hz——计时干净**且无约 5 s 弹出**（肉眼确认）。仅 androidx.webgpu / SF / OEM VRR 本身是流畅的。 | 弹出是 Wasmtime/`host-dawn` 特有：重开 D2（CM 泵 vsync→present 延迟）与 D3（无 present 时间戳）。 |
| D25 | NativeGpu Dawn C + Wasmtime 会流畅（无 androidx JNI） | **Closed** — 同一类约 5 s | 2026-09-01 `fullscreen-surface` + `libwebgpu_dawn.so`：`dlopen` 成功，Vulkan adapter，Choreographer 120 Hz 0 `>20ms`，画面回退仍在。ART/Kotlin 热路径税对这次抖动是 **Closed**。 | 转移表：[`gfx-hitch-native-dawn.zh.md`](gfx-hitch-native-dawn.zh.md)。 |

### 6.5 建议探针（一次一个变量）

1. **`dumpsys SurfaceFlinger --timestats`** 对着 SurfaceView BLAST 层 — `presentToPresent` 直方图 vs 应用 acquire 直方图。  
2. **Perfetto FrameTimeline** 采 20 s：`BUFFER_STUFFING` / `PREDICTION_ERROR` 是否对齐弹出。  
3. **系统设置锁 120 Hz**（不改 app 投票）。  
4. **每 N 帧 skip-present**（guest 或 host）抽干 BLAST — 仅当 timestats 显示 stuffing。  
5. **无 Wasmtime 原生立方体** A/B。若同样抖就停在合成器。 → **已做**（D24）：原生立方体流畅（无约 5 s 弹出）→ 抖动是 Wasmtime/`host-dawn` 特有，不是上游。重开 D2/D3。  
6. **Dawn C NativeGpu A/B**（去掉 androidx JNI，保留 Wasmtime）。 → **已做**（D25）：同样弹出 → 不是 androidx 门面。转到 [`gfx-hitch-native-dawn.zh.md`](gfx-hitch-native-dawn.zh.md)。
