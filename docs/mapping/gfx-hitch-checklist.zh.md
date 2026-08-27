# 立方体连续 present 抖动排查表

[English](gfx-hitch-checklist.md) | **中文**

不是切刀队列。真机调查：仓外旋转立方体（Vivo V2458A，Android 16，`arm64-v8a`，Mali-G925-Immortalis MC12，120 Hz）。连续 `wasi-gfx` `on-frame`，不是 GFXV 仪器那 500 ms。不要 vendor demo。不要给上游提 GitHub issue。

下列 **Check** 以本页中文表为准；英文页与本页同步。Draft PR 保持打开直到画面不再抖。

相关：[`gap-webgpu-wit-androidx.zh.md`](gap-webgpu-wit-androidx.zh.md) §5、[`threading-android.md`](threading-android.md) §8、[`frame-loop-suggestion.zh.md`](frame-loop-suggestion.zh.md)。Guest 时钟 / dt 在**仓外** examples 仓。

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
| C2 | 在当帧 present / 下一 acquire / 纯 CPU 帧环上 `close()` | **Closed**（UAF） | 立刻 close → `0x20` / `0x1f8`。无 fence 的 keep-last-N → ~45 s 崩。须 keep 3 **且** GPU 完成。 |
| C3 | Host 把 `postGfxVsync` 锁成 ~60 Hz | **Mitigated** | 已去掉 `MIN_GFX_VSYNC_NS`。开局过快消失；仍抖。该 cap 本身会在 120 Hz 上造成隔拍抖动。 |
| C4 | Guest 每拍 `angle += const` | **Mitigated** / **Guest** | 立方体已用 `wasi:clocks/monotonic-clock#now`（rAF 式 dt，50 ms 钳制）。Pin `frame-event` 仍是 `{ nothing: bool }`。 |
| C5 | 吞掉帧中途到达的 vsync | **Mitigated** | Native `GfxOnFrameGate`：`pending \|\| in_frame` 时 `post` 丢拍。仍抖 → 不是唯一原因。测试 `wasi_gfx_frame_loop_vsync_paced` 已更新。 |
| C6 | GFXV 仪器本身 | **Closed** | `CLOSE_AFTER_VSYNC_MS = 500`；从未看到数秒 present。Cpu 回收：`WasiWebGpuCanvasContextFrameLifetimeInstrumentedTest`。 |

## 2. 剩余原因（按此顺序 check）

| ID | 假设 | Check | 本轮如何核对 | 仍 Open 则下一步 |
|----|------|-------|--------------|------------------|
| H1 | vsync 之后、`get-current-texture` **等待上一帧 GPU**（`awaitCanvasGpuDone`），present 相对 scanout 偏晚 | **Likely** | `DawnWasiWebGpuHost.canvasContextGetCurrentTexture` 在 acquire **之前**等 `lastCanvasSubmitDone`。Guest 在该等待**之前**采样 `now`，然后很晚才 present。相位随 GPU 时间漂。 | 把 fence 等待移出 vsync→present 路径；UAF 环保留。真机打 vsync→present ns。 |
| H2 | `in_frame` 丢拍 + 120 Hz：帧时在 8.3 ms 两侧振荡 → **60/120 抖动** | **Likely** | 一帧 7 ms 则吃下一拍（120 Hz）；10 ms 则该拍已丢、再等 ~16.6 ms（60 Hz）。立方体 + JNI + fence 泵可能卡在边界。 | 钉死相位（总等**下一** Choreographer，或计数拍）。打 consume 间隔直方图。 |
| H3 | `clocks.now` 是进程 `Instant`，不是 `Choreographer.frameTimeNanos` | **Open** | Host `monotonic-clock.now` = `Instant::now` 流逝 ns（`native/src/cm.rs`）。Guest dt 是「GpuThread 醒来时」，不是显示器 vsync id。小噪声 → 微抖；大跳多半是 H1/H2。 | 等 WIT `frame-event` 带 instant（钉变更 — 不在本 PR）。 |
| H4 | `queueWriteBuffer` **每次 `ByteBuffer.allocateDirect`** | **Likely** | `DawnWasiWebGpuHost.queueWriteBuffer` 每次 uniform 上传都分配 direct buffer（立方体 64 B/帧）。GpuThread 上分配器 / GC 卡顿。 | 复用 direct buffer；`synchronized(gpuLock)`。 |
| H5 | `queueWriteBuffer` **不拿 `gpuLock`**，而 `eventPoller` 在该锁下 `processEvents` | **Open** | 写路径与 5 ms poller 无同步。Mali 上并发 Dawn 是已知 SIGSEGV 类，也可造成卡顿。 | `writeBuffer`（及其它未加锁 Dawn 调用）持 `gpuLock`。 |
| H6 | `PresentMode.Fifo` + `GPUSurface.present()` **在此 AAR 上不等** GPU/合成器 | **Open** | configure 写死 `PresentMode.Fifo`（`DawnWasiWebGpuHost` ~753）。真机：Fifo `present()` 在 GPU 完成前就返回（过早 close 会 UAF）。CPU present 相位可与 BLAST 不一致。 | 看 `getCapabilities().presentModes`；若有 Mailbox 可试。不要假定 Fifo = 等 vsync。 |
| H7 | 不在 `awaitCanvasGpuDone` 时，fence 被 **`POLL_MS = 5`** 推迟 | **Mitigated**（等待路径）/ **Open** | `awaitCanvasGpuDone` 每 1 ms 泵 `processEvents`。后台 poller 仍 sleep 5 ms。其它回调仍可能抖 5 ms。 | canvas fence 在飞时泵或缩短 poll。 |
| H8 | WG-6 **`queue.submit` 自动 present** 后再 guest `context.present` | **Closed**（双 present） | `presentPendingCanvasFrameLocked` 幂等（pending 已清）。第二次 present 是 no-op。不是每帧两次 present。 | — |
| H9 | SurfaceView / BLAST 额外缓冲 vs Choreographer | **Trace** | UI `Choreographer.doFrame` 推 gate；SurfaceView 合成是第二座钟。本轮无 systrace。 | `atrace` gfx/view + `Choreographer` vs `GPUSurface.present` 时间戳。 |
| H10 | GpuThread 上 CM `stream.read` / `callRunConcurrent` 成本 | **Open** | Pin `on-frame` 是同步 `func`；`poll_produce` **阻塞** CM 驱动（无 stackful async）。与 present 同线程的额外工作。 | 测 read+host import ns；未做。 |
| H11 | 每帧 guest `resource.drop`（texture/view/encoder/cb） | **Open** / **Guest** | CM dtor 只动表；Dawn `close` 在 host 环。每帧额外 JNI/CM。 | 推迟 drop；host 已回收交换链。 |
| H12 | guest 慢时 1-slot 丢拍（`pending` 已占） | **Mitigated** 与 C5/H2 重叠 | 未消费拍仍丢。若 guest 经常超过一拍，节奏是 floor(refresh / N)。 | 同 H2 直方图。 |
| H13 | Guest dt **50 ms 钳制** | **Closed**（这次抖动） | 只在停顿 ≥50 ms 时钳。用户看到的是频繁小顿，不是 50 ms 跳。 | — |
| H14 | 首帧 dummy **1/60 s** dt（`last_ns == 0`） | **Closed**（仅开局） | 一帧。用户抖动是连续的。 | — |
| H15 | 每帧上传 **列主序 mat4** `write_buffer` | **Open** | WebGPU 正确；成本主要是 H4 分配而非 64 B 拷贝。 | 并入 H4。 |
| H16 | `GpuThread`（`HandlerThread`）上 ART GC / JIT | **Trace** | 旧崩溃栈见过解释器（`ArtInterpreterToInterpreterBridge`）。Debug 更抖。 | Release minify；`atrace` am。 |
| H17 | submit 之后 guest 仍调 `canvasContextPresent` | **Closed** | 同 H8；no-op。 | — |
| H18 | Depth / MSAA / 过大交换链 | **Closed** | 立方体在 `depth24plus`→RGBA8 事实后跳过 depth。交换链是窗口大小。 | — |
| H19 | 两个 Choreographer 回调（`doFrame` 里 `postFrameCallback`） | **Closed** | 标准 vsync 注册；一条回调链。本身不是双拍。 | — |
| H20 | 产品 linker 缺 `monotonic-clock.now` | **Closed** | 已在 `native/src/cm.rs` 注册；立方体 import 了。 | — |
| H21 | `GPUSurface.getCurrentTexture` 持 `gpuLock` 且可能在 Dawn/BLAST **里阻塞** | **Likely** | `surfaceGetCurrentTexture` 在锁内调 `getCurrentTexture()`。BLAST 无空闲图则 GpuThread 卡住，poller 也无法 `processEvents`。叠在 H1 上（先等 GPU，再可能等 acquire）。 | 打 acquire ns 与 `SurfaceGetCurrentTextureStatus`；Timeout/Suboptimal 即抖动信号。 |
| H22 | `onSubmittedWorkDone` executor 是内联 `Runnable::run` | **Closed**（不是第二座钟） | `callbackExecutor = Executor(Runnable::run)`：fence 跑在调用 `processEvents` 的线程（GpuThread 泵或 poller）。不是双 vsync。与未加锁 `writeBuffer` 并发 close 见 H5。 | 并入 H5。 |
| H23 | UI `Choreographer` → JNI `postGfxVsync` → GpuThread condvar | **Open** | `Store.postGfxVsync` 在 UI 线程 JNI；`wait_take` 在 GpuThread。通常亚毫秒；UI 卡会推迟拍。 | systrace `Choreographer#doFrame` vs native `post`。 |
| H24 | 窗口 / Surface 刷新率 ≠ Choreographer 120 Hz | **Open** / **Guest** | Host 写死 `PresentMode.Fifo`（H6）。仓外 `SurfaceView` 可能偏 60 Hz，而 `doFrame` 是 120。 | examples 仓：`Surface.setFrameRate` / display mode。打 present vs `frameTimeNanos` Δ。 |
| H25 | `deviceGetQueue` **每次 insert 新** HandleTable `Queue`，且无 `gpuLock` | **Open** | `DawnWasiWebGpuHost.deviceGetQueue`。仅当 guest 每帧 `get-queue` 时表会涨。立方体应缓存。 | host intern 一个 queue handle；持 `gpuLock`。 |
| H26 | `awaitCanvasGpuDone` 泵 `processEvents` 时 `tryDrop` 旧交换链 | **Open** | fence 回调 `retireGpuDoneCanvasFramesLocked` 可跑在 vsync→present 的泵上。Mali `GPUTexture.close()` 会卡住这条路径。 | 只在 present 之后的 poller 上 retire，不要在 acquire 等待中 close。 |
| H27 | Android 16 VRR / 显示模式切换 | **Trace** | V2458A 能 120 Hz；活动模式可能变。 | `dumpsys display`；systrace。 |
| H28 | 每帧 `textureCreateView` / encoder / bind-group 的 CM+JNI | **Open** | Host `textureCreateView` 与 `deviceCreateCommandEncoder` **有** `gpuLock`（不像 H4）。成本是 JNI/CM，不是 Dawn 竞态。与 H10/H11 重叠。 | 测量；推迟 guest drop（H11）。 |

## 3. 建议杀序

1. **H1 + H26** — 不要在 vsync 与 present 之间等 GPU（或 close 旧 BLAST 图）；UAF 环保留。  
2. **H21** — 打 acquire 耗时/状态；若 AAR 允许，不要在阻塞的 `getCurrentTexture` 上持 `gpuLock`。  
3. **H4+H5** — `writeBuffer` 加锁 + 复用 direct buffer。  
4. **H2** — 真机 `on-frame` consume 间隔直方图（8.3 vs 16.6 ms）。  
5. **H6+H9+H24** — present-mode + Surface 帧率 vs Choreographer systrace。  
6. **H3** — 等 WIT `frame-event` instant，或仅 host 侧时钟（不要再加一套 WIT）。

## 4. 本分支已改

- 产品 canvas 交换链环 + 异步 `onSubmittedWorkDone` + 下一 acquire 等**上一帧** fence。  
- 去掉 60 Hz `postGfxVsync` cap。  
- `native/src/host.rs` 里 `in_frame` vsync 丢拍。  
- Cpu host pending/last canvas 回收；帧寿命仪器。  
- 仓外立方体：`clocks.now` dt（不在本 git 树）。

## 5. 真机行

| 设备 | C1–C5 之后仍抖 | 备注 |
|------|----------------|------|
| Vivo V2458A（PD2415 / V2458A），Mali-G925，120 Hz | **是** | 崩溃已消失；视觉抖动仍在 2026-08-27。 |
