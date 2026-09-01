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
| H27 | Android 16 VRR / 显示模式切换 | **Closed**（app 侧） | 强行 SF 120 → 抖 **约 3 s**（已撤回）。`CHANGE_FRAME_RATE_ONLY_IF_SEAMLESS` **仍约 5 s**。弹出时 Choreographer/acquire 仍 8.3 ms。`mAlwaysRespectAppRequest=false`；Vivo `vivo_rms_screen`。**D22 2026-09-01：**系统设置 `min_refresh_rate=120` + `vivo_screen_refresh_rate_mode=120`（不改 app 投票）。 | 不要再叠 DisplayManager / GameState。设备上仍锁着；恢复：`settings delete system min_refresh_rate` 且 `vivo_screen_refresh_rate_mode=1`。 |
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

**筛选。** 约 5 s 那一下发生时，`FullscreenSurface` Choreographer 和 `GfxHitch` acquire 仍约 8.3 ms（`>20ms=0`）。凡是会 **卡住** `run_concurrent`、GpuThread、MoonBit GC、或 Wasmtime epoch/fuel 中断的路径，都会在 `lastDtNs` 上冒尖。这些对这次抖动是 **Closed**。剩下的只能是：(a) 应用仍在按时 present，SurfaceFlinger **回退积压的 BLAST 图**；或 (b) 不误 app vsync 的 OEM 策略。**D24 修正（2026-08-29）：**纯 androidx.webgpu 原生立方体计时同样干净但**无约 5 s 弹出**，因此 (a)/(b) 单独不构成抖动——它是 Wasmtime present 路径特有。**D25（2026-09-01）：**Dawn C NativeGpu（无 androidx JNI）仍弹出 → ART/Kotlin 不是放大器。hitch 分支随后量化了 **D2 相位**（JNI 路径上约 19% 的 vsync→present 跨过 8.3 ms 拍）并 **Confirmed D3**（无 present 时间戳；SF stuffing）。D25 之后剩下的链是 guest + CM 泵 + 无时间戳的 `present` + BLAST。转移表：[`gfx-hitch-native-dawn.zh.md`](gfx-hitch-native-dawn.zh.md)。

依据：本树（`native/src/engine.rs`、`cm.rs` 的 `poll_produce` / `nativeCallRunConcurrent`、host-dawn present、仓外 `run.mbt`）；[AOSP frame pacing](https://source.android.com/docs/core/graphics/frame-pacing)（SF 等到同相位才 latch）；[AOSP 游戏循环 / buffer stuffing](https://android.googlesource.com/platform/docs/source.android.com/+/master/en/devices/graphics/arch-gameloops.html)；[Perfetto FrameTimeline](https://perfetto.dev/docs/data-sources/frametimeline)（`BUFFER_STUFFING`、`PREDICTION_ERROR`「周期性自校正」）；[Wasmtime 中断执行](https://docs.wasmtime.dev/examples-interrupting-wasm.html)（epoch/fuel — **本 Engine 未开**）。不要给上游提 GitHub issue；Android 事实只记本仓。

### 6.1 运行时接线

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| D1 | 同步 pin `on-frame` + 阻塞 `poll_produce`（不用 `Poll::Pending`） | **Closed**（这次 5 s 抖动） | `GfxOnFrameProducer` 在 GpuThread 上 condvar 等待：pin 是同步 `func`，WAT 在 stream.read `BLOCKED` 会 trap。这是 **每一拍**，也是 acquire 8.3 ms 的原因，不是 5 s 定时器。 | stackful CM async 是以后的产品事，不是这次弹出。 |
| D2 | UI vsync → JNI `postGfxVsync` → 8MiB `wasmtime-cm-pump` → Dawn present | **Likely** — present **相位**（不是卡住） | hitch 分支：逐帧 `vsync→present`（`System.nanoTime − frameTimeNanos`）。原生 androidx 立方体：**1.2–5.6 ms，0% >8.3 ms**（0/4920）。Wasmtime/`host-dawn`：**1.0–9.3 ms，约 19% >8.3 ms**（1427/7320）。acquire **间隔**仍是 8.3 ms——所以 D2 曾被误判成卡住型 Closed。拆分：`wakeA` 0.5–1.3 ms 稳定（condvar 没问题）；真实工作 ≈ wakeA 1 + guest 2.8（MoonBit async/WIT-ABI 2.6 + 矩阵 0.2）+ host import 3.3 ≈ **7.1 ms < 8.33 ms 拍**。先前 ~5 ms「guest 计算」是 `frames.read(1)` 等 vsync 的伪影（~2.0 ms）。`queueSubmit` 1.4 ms 是自动 `present` ~0.85 ms（H8），不是异常的 submit。**D25：**砍掉那 3.3 ms JNI import **消不掉**弹出。 | Dawn C：NativeGpu `GfxHitch` 的 `present n=` `>8.3ms=`（N4/D2）。guest/host 微优化消不掉重播；D3 是合成器那一半。**P2/P3（2026-09-01，95 s / 12000 帧，Dawn C）：**present `phase-crossing` **2**（margin −0.4 / −2.0 ms，**间隔 55.5 s**，极慢拍漂移）；retire 存活 `<8.3ms=0` 全程为 0。CM 泵锁相 + host present/buffer 生命周期**干净** —— 出阵路径不是这次抖动。 |
| D3 | Dawn `present()` **没有** presentation timestamp（无 Swappy / `eglPresentationTimeANDROID`） | **Confirmed**（静态）+ **Mitigated**（2026-09-01 修复） | Host 从不设目标 present 时刻。hitch 分支：`libwebgpu_c_bundled.so` 的 `llvm-readelf`/`nm` — NEEDED 仅 android/log/dl/c/m，**无 Swappy**，无 `eglPresentationTimeANDROID`。SF timestats：`appBufferStuffingJankyFrames` 主导（60 档 21961 + 90 档 34667；`sfPredictionErrorJankyFrames` 4247）。20 s Perfetto 显示级 **2407/2407 On-time**。vendor `debug.sf.disable_backpressure=1`。回退在 layer/buffer，不在显示。D25：Dawn C `wgpuSurfacePresent` 仍无时间戳。**Dawn C timestats 2026-09-01 约 45 s**（先 `--timestats -clear -enable` 再 `-dump`；光 `--timestats` 是空的）：SurfaceView BLAST **5** + 1 VRI。Timeline `appBufferStuffingJankyFrames=0`，因为 `totalTimelineFrames=0`（SurfaceView 不进 FrameTimeline）。遗留直方图仍是 stuffing **行为**：`desired2present` / `acquire2present` 双峰约 18–19 ms vs 约 28–30 ms（约 41% 多一拍）；`present2present` 8 ms=5398 / 33 ms=1；`present2presentDelta` 25 ms=2。**2026-09-01 修复：**`wgpuSurfacePresent` 前立刻 `ANativeWindow_setBuffersTimestamp`（默认超前 **2** 拍；`debug.wasmtime.gfx.desired_present_beats` / `WASMTIME_GFX_DESIRED_PRESENT_BEATS`，`0` 关）。V2458A `rc=0`。约 59 s 1:1 且 D22 锁着：`desired2present` **5 ms=7072**（曾钉在约 28–30 ms）；`present2presentDelta` 25 ms=0；`averageFPS` 125.062（skip `n=6` 是 104）。 | A/B：`desired_present_beats 0`。skip-present 仍是探针。不要 Mailbox。 |
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
| D13 | Guest 转角 vs 合成器回退 | **Closed**（keep-6 加重）/ **Open**（肉眼） | SF 重播 BLAST *n−2* 时，立方体 **看起来** 往回走，即使 `angle` 已经加过。D3 时间戳抽干了 `desired2present`（约 29→5 ms），弹出仍在。keep-6 **提高**了抖动频率；SurfaceView BLAST 仍 **5**。已退回 keep-3。与 keep-8 / H27 同类。**P4（2026-09-01，95 s / 12720 帧）：**guest 转角时钟（Choreographer `frameTimeNanos` 逐帧差分）`angleDt 8-9ms=全部`、`9-17ms=0`、`>17ms=0`——**变换矩阵在时间上严格线性**，无跳拍无回退。**N9（2026-09-01）：**present 提交细节（configure / Dawn commit / 65 s SF timestats）干净——Dawn C 路径上无 rewind/drop/jank。 | 不要再叠 keep。 |

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
| D19 | 5 张 BLAST BufferQueue stuffing；SF 重播再抽干 | **Mitigated**（时间戳） | 同 D3。hitch 分支 timestats 已确认 stuffing 是主导 *jank 类*。Dawn C：Timeline 类 **n/a**（`totalTimelineFrames=0`）；`desired2present` 双峰曾是 stuffing 信号。skip `n=6` 抽干多出的一拍（FPS 约 104）。**时间戳 2 拍**把 `desired2present` 抽到约 5 ms，FPS 仍 125。 | skip 仍是探针。不要 Mailbox。 |
| D20 | SF「等到同相位」+ UID 被节流到 120 的约数（60/90） | **Closed**（2026-09-01） | `dumpsys SurfaceFlinger`（V2458A，Android 16）：`GameFrameRateOverrides=` 空（无 GameManager/RMS UID 映射）；`setFrameRate=(uid, frameRate)={10504, 120.00 Hz}`；SurfaceView BLAST 层 `requestedFrameRate: {120.00 Hz ExactOrMultiple}`；`idleScreenConfig timeoutMillis:-1`；该层 `FPS ring buffer` 全程 120.39/120.31/120.53…（无 60/90 档）。SF 未给本 UID 降帧到 120 的约数。 | 不要再设 GameState / DisplayManager 投票。 |
| D21 | VSyncPredictor `PREDICTION_ERROR`，调度器「周期性校正」 | **Closed**（这次抖动） | 预测器 recovery **关着**（`vsync_predictor_recovery: false`；`compositionStrategyPredictionState=DISABLED`）。hitch 分支显示级 Perfetto 2407/2407 On-time。Dawn C 2026-09-01 约 22 s：**2155/2155** 实际显示帧 `JANK_NONE` / `PRESENT_ON_TIME` / `PREDICTION_VALID`；**0** `JANK_PREDICTION_ERROR`。SurfaceView BLAST **不发** surface-frame timeline（Perfetto：SurfaceView 未支持；与 D3 `totalTimelineFrames=0` 一致）。BLAST 塞满回退不会表现为 prediction-error jank。 | — |
| D22 | Vivo RMS / `vivo_rms_screen` / 自适应 60·90·120 | **Closed**（Hz 掉档不是剩下的原因）/ **Mitigated**（本窗口系统锁） | 设备有 `vivo_rms_screen`；`mAlwaysRespectAppRequest=false`；强行 SF 120 **提高** 抖动频率（H27）。**2026-09-01 系统设置锁**（不改 app 投票）：`min_refresh_rate` null→`120.0`，`vivo_screen_refresh_rate_mode` 1→120，`peak_refresh_rate` 仍 120。DisplayModeDirector `PRIORITY_USER_SETTING_MIN_RENDER_FRAME_RATE` 0→120。约 60 s 1:1 TimeStats 对照 D3 智能切换基线：`refreshRateSwitches=0`；`averageFPS=125.007`；`present2present` 8 ms=7124（无 16/25/33 ms）；`present2presentDelta` 25 ms **0**（曾为 2）；`desired2present` **单峰**约 28–30 ms（曾双峰 18–19 vs 28–30）。BLAST 仍 **5**。`GfxHitch` `lastDtNs` 仍约 8.3 ms。锁把 stuffing 深度钉住；并没有抽干多出的一拍（那是 skip `n=6`）。 | 时间戳已落地（D3）。不要再叠投票。恢复：`settings delete system min_refresh_rate`；`settings put global vivo_screen_refresh_rate_mode 1`。 |
| D23 | Kernel / DisplayModeDirector 空闲降刷新超时 | **Open**（弱） | 立方体每拍 present，图层并不空闲。厂商 SF 里仍常见数秒 idle timer。 | 打开「显示刷新率」叠加层，看弹出时 Hz 是否掉。 |
| D24 | 不含 Wasmtime 的原生 androidx.webgpu 立方体也会同样抖 | **Closed** — 上游单独**不会**复现 | `hosts/native-webgpu` `CubeActivity`（纯 androidx.webgpu 1.0.0-alpha05，无 Wasmtime、无 `host-dawn`）在 V2458A 上：1:1 出帧 @120 Hz，Choreographer ~8.3 ms（0 `>20ms`），acquire ~8.3 ms（0 `>20ms`，`SuccessOptimal`），显示钉在 120 Hz——计时干净**且无约 5 s 弹出**（肉眼确认）。仅 androidx.webgpu / SF / OEM VRR 本身是流畅的。 | 弹出是 Wasmtime/`host-dawn` 特有：重开 D2（CM 泵 vsync→present 延迟）与 D3（无 present 时间戳）。 |
| D25 | NativeGpu Dawn C + Wasmtime 会流畅（无 androidx JNI） | **Closed** — 同一类约 5 s | 2026-09-01 `fullscreen-surface` + `libwebgpu_dawn.so`：`dlopen` 成功，Vulkan adapter，Choreographer 120 Hz 0 `>20ms`，画面回退仍在。ART/Kotlin 热路径税对这次抖动是 **Closed**。 | 转移表：[`gfx-hitch-native-dawn.zh.md`](gfx-hitch-native-dawn.zh.md)。 |

### 6.5 建议探针（一次一个变量）

1. **`dumpsys SurfaceFlinger --timestats`** — hitch 分支 **已做**（androidx，stuffing *类*）。Dawn C 2026-09-01 **已做**：必须先 `-clear -enable` 再 `-dump`（光 `--timestats` 是空的）。Timeline 类为 0；`desired2present` 双峰约多一拍。
2. **Perfetto FrameTimeline** 约 20 s — hitch 分支 **已做**（显示级全 On-time）。Dawn C 2026-09-01 **已做**：2155/2155 显示 On-time + Valid；0 PredictionError；SurfaceView layer timeline n/a。
3. **系统设置锁 120 Hz**（不改 app 投票）。 → **已做**（D22 2026-09-01）：`min_refresh_rate=120` + `vivo_screen_refresh_rate_mode=120`。本约 60 s 里 25 ms 回退间隔 2→0；`desired2present` 钉在塞满的约 29 ms。BLAST 仍是 5。
4. **每 N 帧 skip-present** — **已做** `n=6` A/B：`desired2present` 多一拍的峰消失；25 ms `present2presentDelta` 2→0。闸门默认关（`debug.wasmtime.gfx.skip_present_n`）。不要 Mailbox。
5. **无 Wasmtime 原生立方体** A/B。 → **已做**（D24）：流畅。抖动是 Wasmtime 路径特有。
6. **Dawn C NativeGpu A/B。** → **已做**（D25）：同样弹出 → 不是 androidx 门面。
7. **NativeGpu 上 N4 + D2 直方图**（`GfxHitch` acquire 间隔 + `vsync→wgpuSurfacePresent`）。 → **已做**（2026-09-01 Dawn C）：acquire `>20ms=0`；`>8.3ms=0`（对照 hitch JNI 路径 19%）。卡住型 Closed；跨拍相位在本样本上 Closed。BLAST 仍是 5。

### 6.6 瓶颈落点（hitch 分支数字 × D25）

不是单一热点。工作量能塞进拍内（JNI 路径约 7.1 ms / 8.33 ms）。两级：

1. **相位源（D2，链路中段）。** CM 泵把 guest 执行串进 present deadline。condvar（`wakeA`）稳定。D25 否掉「15 次 GPU JNI 是这次 5 s 弹出的必要条件」。剩下的抖动是 guest async/WIT-ABI。**P2/P3（2026-09-01，95 s / 12000 帧）：**`phase-crossing` 2（间隔 55.5 s）；retire `<8.3ms=0`。**P4（2026-09-01，95 s / 12720 帧）：**guest 转角时钟逐帧差分 `angleDt 8-9ms=全部`、`9-17ms=0`、`>17ms=0`——**变换矩阵时间严格线性**。**P5（2026-09-02，约 12 min）：**`take-skip` **0 次**——guest 的 `now`/`angle` 每拍 8.33 ms 步进，**与 present 相位解耦**。**相位补充（2026-09-02）：**重读完整 `phase-crossing` 序列发现 present 相位会**偶发失锁（snap）**——23:44 一次 `lat` 5→9.6 ms、`margin` +3→−1.3 ms 一拍内翻负、此后 `cross` 每秒 +2~5；但 30 分钟仅见一次，且 P5 证实 snap **不传递到 guest 内容**（take-skip 仍 0）。出阵路径（CM 泵相位 + present/buffer 寿命）与 guest 内容**都干净**。
2. **变成回退的地方（D3/D19，链路末端）。** **Mitigated：**`ANativeWindow_setBuffersTimestamp`（默认 2 拍）把 `desired2present` 从约 29 ms 降到约 5 ms，仍 125 fps。BLAST 仍是 5。若还有画面回退，见 D13。**D20 已 Closed（2026-09-01）：**SF 无 GameFrameRateOverrides、UID 明确 120 Hz、idle timeout -1、FPS ring 稳定 120——SF 侧无降帧。**N9（2026-09-01）：**present 提交细节本身干净 — `surface-caps formats=[22,23,40,30] present=[4,1] alpha=[4]`（RGBA8 + `Fifo(1)` + `Inherit(4)`），configure 已对齐 `caps.alphaModes[0]`，Dawn pin = androidx AAR SHA（同 commit），65 s SF 窗口（`droppedFrames=0`、`jankyFrames=0`、`present2presentDelta 25ms=0`、`desired2present 5ms=7811`）在 Dawn C 路径上**无 rewind/drop/jank**。

**结论（2026-09-02，暂停验证）：**三侧可观测层——guest 内容（P4/P5 线性、与相位解耦）、SF 计数器（`droppedFrames=0`、`jankyFrames=0`、无 rewind/drop）、present 提交（N9 干净）——**全部干净**。视觉「弹」落在**未被计量的层**：最可能是 SF 计数器之外的合成器/面板 BLAST 重播（D13/D19 的 rewind），或偶发 present-phase snap 的 host 侧瞬态——两者都**尚未在弹的瞬间被逐帧抓帧证实**。下一步需事件触发式抓帧（screenrecord 与 `sinceLast` 骤降联动，精确抽取弹前后帧判定方向），或补真实 `onSubmittedWorkDone` fence 关闭 D24 最后一个结构差异。

hitch 分支修复排序经 D25 过滤后：(1) present 时间戳 / `setDesiredPresentTime` — **已落地**（`ANativeWindow_setBuffersTimestamp`，默认 2 拍），(2) draw/present 解耦，(3) 砍 JNI — **已做，不够**，(4) 砍 condvar — 收益小，(5) 简化 guest — 只缩小 D2。skip-present 是 stuffing 探针，不是修复。

Dawn C 活表：[`gfx-hitch-native-dawn.zh.md`](gfx-hitch-native-dawn.zh.md)。
