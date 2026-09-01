# 立方体抖动：Dawn C 对照 androidx.webgpu

[English](gfx-hitch-native-dawn.md) | **中文**

不是切刀队列。不是 native-dawn 消费车道。NativeGpu 接上 `wgpu*` 之后的真机 A/B（2026-09-01）。不要 vendor demo。不要给上游提 GitHub issue。具名 hitch 活（D3 present timestamp / skip-present）仍是具名 —— 见 [`../agent/native-dawn.zh.md`](../agent/native-dawn.zh.md)。

androidx + `host-dawn` JNI 表：[`gfx-hitch-checklist.zh.md`](gfx-hitch-checklist.zh.md)。**C / H / D** 编号沿用，避免重排。Dawn C 新行是 **N\***。下列 **Check** 以本页中文表为准。

同一台设备：Vivo V2458A（PD2415），Android 16，`arm64-v8a`，Mali-G925-Immortalis MC12，120 Hz。Host：仓外 `hosts/fullscreen-surface`。Guest：同一份 MoonBit 旋转立方体。

**Check**

| 标记 | 含义 |
|------|------|
| **Open** | Dawn C 路径上仍可能造成这次抖动；真机尚未证伪 |
| **Likely** | A/B 之后最可疑 |
| **Mitigated** | 不变量已落地；画面仍抖，单独不够 |
| **Closed** | 对**这次约 5 s 弹出**已排除（仍可能是崩溃/泄漏事实） |
| **Trace** | 需要本路径的直方图 / Perfetto / timestats；尚未做 |
| **n/a** | Dawn C 上不存在（只属于 androidx） |

## 0. A/B 三角（2026-09-01）

| 构建 | GPU 消费 | Wasmtime / CM 泵 | 约 5 s 回退 |
|------|----------|------------------|------------|
| `hosts/native-webgpu` `CubeActivity` | 仅 androidx.webgpu JNI | 无 | **无**（D24，2026-08-29 肉眼） |
| `fullscreen-surface` + `host-dawn` | `ExperimentalHostCallbacks` → androidx JNI | 有 | **有**（约 5 s；Choreographer / acquire 仍 ~8.3 ms） |
| `fullscreen-surface` + NativeGpu Dawn C | `dlopen` `libwebgpu_dawn.so` + `wgpu*`（立方体热路径无 androidx） | 有 | **有**（同一观感；Choreographer 120 Hz，0 `>20ms`） |

**筛选。** 绕开 ART / Kotlin / androidx JNI **消不掉**这次弹出。native-dawn「Why」（ART 频繁穿越放大相位抖动）对**这次抖动**是 **Closed**。两边仍共享：guest + `GfxOnFrameGate` / `postGfxVsync` + 8MiB `wasmtime-cm-pump` + hitch 环（keep-3 / Fifo / H8）+ Dawn `present()` **没有** presentation timestamp + SurfaceView BLAST + OEM VRR。

Dawn C 安装上 Choreographer 连续数分钟全是 `<11ms`（与 H23 同形）。这仍然 **关闭卡住型** 原因（CM trap、epoch、MoonBit GC、acquire 等待）。**关不掉**合成器回退积压 BLAST，也关不掉从不抬高 `lastDtNs` 的 vsync→present **相位**。

## 1. 转移表：androidx 行 → Dawn C

无新事实则沿用 androidx 证据。「androidx Check」来自 [`gfx-hitch-checklist.zh.md`](gfx-hitch-checklist.zh.md)。

### 1.1 已关闭或 n/a（列出以免再踩）

| ID | 假设 | androidx | Dawn C | 转移 |
|----|------|----------|--------|------|
| C1 | 泄漏交换链纹理 → SIGSEGV | Closed（崩溃） | keep-3 + 推迟 `wgpuTextureRelease` | **Closed**（崩溃）。立方体跑数分钟无 `0x20`。 |
| C2 | 刚 present 的图立刻 `close()` | Closed（UAF） | 同一不变量 | **Closed**（UAF）。 |
| C3 | Host ~60 Hz `postGfxVsync` cap | Mitigated | 同一 `host.rs` | **Mitigated**。共享。 |
| C4 | Guest `angle += const` | Mitigated / Guest | 同一 wasm | **Mitigated** / **Guest**。 |
| C5 | `in_frame` 时丢 vsync | Mitigated | 同一 `GfxOnFrameGate` | **Mitigated**。共享。 |
| C6 | GFXV 500 ms 仪器 | Closed | 不是这个 host | **Closed**。 |
| C7 | androidx `onSubmittedWorkDone` JNI global-ref 泄漏 | Mitigated（崩溃） | 无 AAR 回调 | **n/a**。不要抄每 2 帧 batch fence。 |
| C8 | `processEvents` vs 无锁 `getCurrentTexture` Mali 竞态 | Open（崩溃） | `process_events` 与 acquire 在**同一**泵线程 | **Closed**（这次竞态）。新的 Dawn 线程问题见 **N3**。 |
| H1 | acquire 等待上一帧 GPU fence | Mitigated | acquire 不等 | **Mitigated**。 |
| H4 / H5 / H15 | 每帧 `ByteBuffer.allocateDirect` / 无锁 `writeBuffer` | Mitigated | guest `list<u8>` → 一次 host copy → `wgpuQueueWriteBuffer`；无 ART | **Closed**（这次抖动）。路径没了；弹出还在。 |
| H8 / H17 / D4 | `queue.submit` 自动 present + guest `context.present` | Closed | 同一 H8 no-op | **Closed**。 |
| H10 / H11 / H28 | 每帧 CM + **JNI** 成本 | Closed（5 s） | CM 仍在；GPU 方法上的 JNI 没了 | **Closed**（5 s）。会是连续抖。 |
| H13 / H14 / D11 | dt 50 ms 钳 / 首帧 1/60 | Closed | 同一 guest | **Closed**。 |
| H16 / D7 | `GpuThread` 上 ART GC / JIT | Closed（5 s；GC ~87 s） | GPU 热路径是 Rust + Dawn C | **Closed**（更强）。A/B：ART 更少，弹出一样。 |
| H18 | Depth / MSAA / 过大交换链 | Closed | 立方体仍跳过 depth | **Closed**。 |
| H19 | 两个 Choreographer 回调 | Closed | 同一 host | **Closed**。 |
| H20 | 缺 `monotonic-clock.now` | Closed | 同一 linker | **Closed**。 |
| H22 | 内联 fence executor 当第二座钟 | Closed | 无 androidx executor | **n/a**。 |
| H23 | UI Choreographer → JNI vsync → GpuThread condvar **卡住** | Closed（5 s） | 同一 vsync JNI；直方图仍 `<11ms` | **Closed**（卡住型）。相位是 **D2**。 |
| H25 | 每次 `device.queue` 新 handle | Mitigated | intern 一条 queue | **Mitigated**。 |
| D1 | 同步 `on-frame` + 阻塞 `poll_produce` | Closed（5 s） | 未改 | **Closed**（5 s）。每拍成本，不是 5 s 定时器。 |
| D8–D10 / D14–D18 | Guest 调度 / Wasmtime epoch / 「CM 慢」 | Closed | guest + Engine 未改 | **Closed**。共享；直方图干净。 |
| N7 | 启动时 `wgpuInstanceWaitAny` 2 s | — | 只在 adapter/device | **Closed**。不是每帧。 |

### 1.2 A/B 之后仍活（按此顺序 check）

| ID | 假设 | androidx | Dawn C | 下一步 |
|----|------|----------|--------|--------|
| **N1** | ART / Kotlin / androidx JNI 热路径就是这次 5 s 弹出 | （playbook Why） | **Closed** | 不要再为这次抖动拆 JNI。 |
| **D2** | vsync → `postGfxVsync` → 8MiB CM 泵 → Dawn present **相位**（`lastDtNs` 不冒尖） | 卡住型 Closed；playbook 仍点名相位 | **Open** / **Likely** | 共享。A/B 否掉了「ART 穿越放大器」。量 vsync→`wgpuSurfacePresent` ns。 |
| **D3** / **D19** | `present()` 无 presentation timestamp；BLAST 塞满；SF 重播 *n−2* | Open / Likely | **Open** / **Likely** | 共享。`timestats` `presentToPresent`；Perfetto `BUFFER_STUFFING`。只有确认 stuffing 才 skip-present。不要 Mailbox。 |
| **D13** | 立方体**看起来**往回走，但 `angle` 已经加过 | Likely（症状） | **Likely** | 用来区分 guest dt 和 D3。 |
| **D20** | SF 等到同相位 + UID FPS 约数 | Open | **Open** | 共享。`dumpsys SurfaceFlinger` 图层 override。**不要** GameState。 |
| **D21** | VSyncPredictor 周期性校正 | Open / Trace | **Open** / **Trace** | 共享。对着弹出采 Perfetto。 |
| **D22** / **H27** | Vivo RMS / 智能刷新；app 投票会加重 | Open / Likely；app 投票 Closed | **Open** / **Likely** | 系统设置锁 120 Hz。不要再叠 DisplayManager 投票。 |
| **D23** | 空闲降刷新超时 | Open（弱） | **Open**（弱） | 弹出时看刷新率叠加层。 |
| **D24** | 无 Wasmtime 的 androidx 立方体同样抖 | **Closed**（并不抖） | 仍是对照 | 流畅基线，保留。 |
| **D25** | Dawn C + Wasmtime 立方体流畅 | — | **Closed**（并不流畅；本页） | 抖动不是 androidx 门面。 |
| **H2** / **H12** | 隔拍 60/120 / 额外等 wait 开始 | Mitigated | 同一 gate | **Mitigated**。用 **N4** 确认 Dawn C acquire 直方图。 |
| **H3** / **D6** / **D12** | `clocks.now` vs Choreographer；pin `frame-event` 无时间戳 | Mitigated | 同一 | **Mitigated**。合成器回退仍会显示旧图。 |
| **H6** / **H9** | Fifo vs Mailbox；BLAST 下限 5 | Mitigated / Closed（app 压不了） | Fifo + 意图 `setBufferCount(4)` | **Mitigated**。用 **N5** 再看 Dawn C 的 gfxinfo BLAST。 |
| **H21** | `getCurrentTexture` 持 `gpuLock` 阻塞 | Mitigated | 无 `gpuLock`；泵上 `process_events` 再 `GetCurrentTexture` | **Mitigated**（锁）。Dawn 内部阻塞见 **N3**。 |
| **H26** / **D5** | poller / keep-3 当成 5 s 周期 | Closed（5 s） | keep-3 仍在；fence **不是** 真 GPU 回调（**N2**） | **Closed**（5 s 周期）。寿命是 **N2**。 |

## 2. 仅 Dawn C 的行

| ID | 假设 | Check | 证据 | 下一步 |
|----|------|-------|------|--------|
| N1 | 立方体 GPU 路径去掉 androidx JNI / ART 就能消弹出 | **Closed** | host + guest + vsync 相同，只换消费；弹出仍在 | — |
| N2 | `queue.submit` 上立刻 `mark_canvas_gpu_done()`；没有 `wgpuQueueOnSubmittedWorkDone` | **Open**（寿命） / **Closed**（5 s 周期） | `native_gpu.rs` 把整环标 `gpu_done` 再 keep-3 retire。androidx 在 poller 上等真 fence。 | 纹理 UAF 再接 C-API work-done。不是 5 s 定时器。 |
| N3 | acquire 上的 `wgpuInstanceProcessEvents`（泵线程）vs androidx poller | **Open** / **Trace** | 与 present 同线程。可能加相位；不应抬高 Choreographer。 | 打 acquire ns + `GetCurrentTexture` 状态（JNI 时是 `GfxHitch`）。 |
| N4 | Dawn C 没有 acquire / present 间隔直方图 | **Trace** | androidx 有 `GfxHitch`。Dawn C logcat 目前只有启动（`dlopen` / adapter / device）。 | NativeGpu 上复用 `<11 / 11–20 / >20ms` 桶。 |
| N5 | `wgpuSurfacePresent` vs androidx `GPUSurface.present` 的 stuffing | **Open** | 都走 `ANativeWindow` → BLAST。Dawn C 尚未重采 gfxinfo。 | `dumpsys gfxinfo` BLAST 张数；对照 5。 |
| N6 | `preferred_canvas_format` → `resolve_device(0)` 第二套 adapter/device | **Closed** | 已复用 guest device（2026-09-01）。是启动 bug，不是 5 s 周期。 | — |
| N8 | `webgpu.h` main 与 Dawn `.so` SHA ABI 不一致 | **Closed**（这次抖动） | adapter/device/present 能跑；立方体在屏上。 | 只有 present 开始报 Error 再重开。 |

## 3. 建议杀序（一次一个变量）

**不要**再叠 DisplayManager / GameState / SurfaceControl 投票。**不要**把再拆一层 JNI 当成下一刀 hitch。

1. **N4** — NativeGpu acquire + present 间隔直方图。若弹出时仍 `>20ms=0`（与 androidx 相同），卡住型保持 Closed。  
2. **D3 / D19 / N5** — Dawn C 的 SurfaceView 上 `dumpsys SurfaceFlinger --timestats` + `gfxinfo` BLAST。  
3. **D2** — vsync `frameTimeNanos` → `wgpuSurfacePresent` 返回（泵线程）。没有直方图空洞的相位。  
4. **D21** — 对着弹出采约 20 s Perfetto FrameTimeline（`BUFFER_STUFFING` / `PREDICTION_ERROR`）。  
5. **D22** — 系统设置锁 120 Hz（不改 app 投票）。  
6. **每 N 帧 skip-present** — 仅当 timestats 显示 stuffing。不要 Mailbox。  
7. **N2** — 仅当 Dawn C 回收时 UAF / SIGSEGV（崩溃车道，不是这次弹出）。

D24 对照仍是无 Wasmtime 的 androidx 立方体。若 D2/D3 卡住，以后可以再做无 Wasmtime 的 **Dawn C** 立方体 A/B；不要 vendor。

## 4. 本路径已保持

- H1 / C2 / H8 / Fifo / intern 一条 queue / keep-3（ND-SURF 不变量）。  
- 不抄 C7 AAR 泄漏 batch。  
- `preferred_canvas_format` 复用 guest device（N6）。  
- 仓外 host：峰值模式 + 同 Hz `setFrameRate`（H24 / H27）。不要再加投票。
