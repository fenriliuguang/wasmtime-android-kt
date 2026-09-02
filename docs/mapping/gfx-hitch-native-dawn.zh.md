# 立方体抖动：Dawn C 对照 androidx.webgpu

[English](gfx-hitch-native-dawn.md) | **中文**

不是 native-dawn 消费车道。抖动剩余队列：[`../agent/gfx-hitch.zh.md`](../agent/gfx-hitch.zh.md)（issue 300；从热路径阶段重开）。NativeGpu 接上 `wgpu*` 之后的真机 A/B（2026-09-01）在 §§0–5 只作**档案**。不要 vendor demo。不要给上游提 GitHub issue。**不要**把 Closed/Likely 当前提。

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

**筛选。** 绕开 ART / Kotlin / androidx JNI **消不掉**这次弹出。native-dawn「Why」（ART 频繁穿越放大相位抖动）对**这次抖动**是 **Closed**。两边仍共享：guest + `GfxOnFrameGate` / `postGfxVsync` + 8MiB `wasmtime-cm-pump` + hitch 环（keep-3 / Fifo / H8）+ SurfaceView BLAST + OEM VRR。Present 时间戳已落地（D3）。keep-6 **加重**了抖动频率（已撤回）。

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
| **D2** | vsync → `postGfxVsync` → 8MiB CM 泵 → Dawn present **相位**（`lastDtNs` 不冒尖） | 卡住型 Closed；hitch 分支 **Likely**（JNI 路径约 19% >8.3 ms） | **Open**（拍内游荡）/ **Closed**（本 25 s 跨拍） | NativeGpu `GfxHitch` 2026-09-01：`present n=2880` `>8.3ms=0`（JNI 路径是 19%）。`lastLatencyNs` 仍 0.8–6.9 ms，在拍内。跨拍相位不是 Dawn C 的故事。剩下 D3。**P2（2026-09-01，95 s / 12000 帧）：**`phase-crossing` **2**（margin −0.4 / −2.0 ms，**间隔 55.5 s**）= 极慢拍漂移（约 1.25 ns/拍），**不是** 约 5 s 周期。CM 泵锁相干净。 |
| **D3** / **D19** | `present()` 无 presentation timestamp；BLAST 塞满；SF 重播 *n−2* | androidx `.so` 上 **Confirmed**（静态 + timestats） | **Mitigated**（时间戳 2 拍） | 共享。**2026-09-01：**`wgpuSurfacePresent` 前 `ANativeWindow_setBuffersTimestamp`（`rc=0`）。默认 2 拍。D22 锁着约 59 s：`desired2present` **5 ms=7072**（曾钉在约 29 ms）；FPS 125.062。skip `n=6` 仍是探针。 |
| **D13** | 立方体**看起来**往回走，但 `angle` 已经加过 | Likely（症状） | **Closed**（keep-6）/ **Open**（肉眼） | 时间戳抽干了 `desired2present`，肉眼仍弹出。keep-6（BLAST+1）**提高**了抖动频率；gfxinfo SurfaceView BLAST 仍 **5**。已退回 keep-3。与 keep-8 抽干池 / H27 投票同类。**P4（2026-09-01，95 s / 12720 帧）：**guest 转角时钟逐帧差分 `angleDt 8-9ms=全部`、`9-17ms=0`、`>17ms=0`——**变换矩阵时间严格线性**。回退只能来自 SF 重播旧 BLAST。 | 不要再叠 keep。下一刀不是更多在途图。 |
| **D20** | SF 等到同相位 + UID FPS 约数 | Open | **Closed**（2026-09-01） | 共享。`dumpsys SurfaceFlinger`（V2458A / Android 16）：`GameFrameRateOverrides=` 空；`setFrameRate UID=10504 → 120.00 Hz`；BLAST 层 `requestedFrameRate 120.00 Hz ExactOrMultiple`；`idleScreenConfig timeout:-1`；层 `FPS ring buffer` 稳定 120。SF 未降帧到 120 的约数。**不要** GameState。 |
| **D21** | VSyncPredictor 周期性校正 | Open / Trace | **Closed**（这次抖动） | `vsync_predictor_recovery: false`。Dawn C 22 s Perfetto：**2155/2155** 显示帧 `NONE` / `ON_TIME` / `VALID`。0 `PREDICTION_ERROR`。SurfaceView BLAST **没有** surface-frame timeline（与 D3 同 n/a）。 |
| **D22** / **H27** | Vivo RMS / 智能刷新；app 投票会加重 | Open / Likely；app 投票 Closed | **Closed**（Hz 掉档）/ **Mitigated**（系统锁） | **2026-09-01：**`min_refresh_rate=120` + `vivo_screen_refresh_rate_mode=120`。本约 60 s `present2presentDelta` 25 ms 2→0；`desired2present` 钉在约 29 ms（曾双峰）。BLAST 仍 5。不要再叠投票。 |
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
| N2 | `queue.submit` 上立刻 `mark_canvas_gpu_done()`；没有 `wgpuQueueOnSubmittedWorkDone` | **Open**（寿命） / **Closed**（5 s 定时器） | 整环标 `gpu_done` 再 keep-3 retire。keep-6 **加重**抖动频率（已撤回）。 | 不要再加 keep。仅 UAF 时再接 C-API work-done。**P3（2026-09-01，95 s / 12000 帧）：**retire 存活 `<8.3ms=0` 全程为 0；retire 落在 24–28 ms（约 3 拍）。无合成前回收 —— `gpu_done` 的「谎言」**没有**在 SF 下复用 buffer。 |
| N3 | acquire 上的 `wgpuInstanceProcessEvents`（泵线程）vs androidx poller | **Closed**（本 25 s）/ **Open**（偶发警告） | 与 present 同线程。开局一次 acquire 警告 `2031693ns status=1`；之后 acquire `last` 0.07–0.8 ms，`>20ms=0`。 | — |
| N4 | Dawn C acquire / present 间隔 + vsync→present 直方图 | **Closed**（卡住型） | 2026-09-01 V2458A，`fullscreen-surface` NativeGpu，约 25 s：Choreographer 3000 `<11ms` `>20ms=0`。Acquire n=3000 `<11ms=2998` `11-20ms=2` `>20ms=0` `status=1`。Present 间隔 n=2880 `<11ms=2867` `11-20ms=13` `>20ms=0`。 | 本窗口卡住型保持 Closed。 |
| N5 | `wgpuSurfacePresent` vs androidx `GPUSurface.present` 的 stuffing | **Mitigated**（时间戳）/ Timeline **n/a** / skip **已抽干** | gfxinfo：SurfaceView **5** 张 BLAST Consumer（与 H9 同下限）+ 1 张 VRI。2026-09-01 1:1 TimeStats 约 45 s：`desired2present` 双峰约 18–19 vs 约 28–30 ms（约 41% 多一拍）；`present2presentDelta` 25 ms=2。skip `n=6`：FPS 104.194；28–30 ms 峰 **0**。**D22 锁：**钉在约 29 ms。**时间戳 2 拍：**`desired2present` **5 ms=7072**；FPS 125.062；25 ms 间隔 0。 | 默认时间戳开。skip 探针关。不要 Mailbox。 |
| N6 | `preferred_canvas_format` → `resolve_device(0)` 第二套 adapter/device | **Closed** | 已复用 guest device（2026-09-01）。是启动 bug，不是 5 s 周期。 | — |
| N8 | `webgpu.h` main 与 Dawn `.so` SHA ABI 不一致 | **Closed**（这次抖动） | adapter/device/present 能跑；立方体在屏上。 | 只有 present 开始报 Error 再重开。 |
| N9 | Dawn C `wgpuSurfacePresent` vs androidx `GPUSurface.present` 的提交细节 | **Closed**（2026-09-01，非差异源） | `surface-caps formats=[22,23,40,30] present=[4,1] alpha=[4]` → RGBA8 为首、`Fifo(1)`+`Mailbox(4)`、`Inherit(4)`。`AlphaMode_Auto` 在 native 默认落到 `Inherit`，故把 configure 对齐到 `caps.alphaModes[0]` 是无操作。`PRESENT_FIFO=0x0001` 正确。Dawn pin = androidx AAR SHA（同 commit）。65 s / 7835 帧：`droppedFrames=0`、`jankyFrames=0`、`present2presentDelta 25ms=0`、`desired2present 5ms=7811`。 | 仅剩差异：D24 每帧真实 `onSubmittedWorkDone` fence vs NativeGpu 仅 mark+retire；NativeGpu 在 `queue.submit` 内自动 present（H8）。 |

## 3. 建议杀序（一次一个变量）

**不要**再叠 DisplayManager / GameState / SurfaceControl 投票。**不要**把再拆一层 JNI 当成下一刀 hitch。

1. **N4 + D2** — **已做**（2026-09-01 约 25 s）：acquire `>20ms=0`；`vsync→present` `>8.3ms=0`（JNI 路径约 19%）。卡住型 Closed。本样本跨拍相位 Closed。  
2. **D3 / D19 / N5** — timestats **已做**（2026-09-01）。BLAST **5**。Timeline 类 n/a；`desired2present` 双峰 = 队列里多一拍。  
3. **skip-present `n=6` A/B** — **已做**：多一拍的峰被抽干；25 ms 回退间隔消失。属性默认 **关**（不是产品 100 fps）。不要 Mailbox。  
4. **D21** — **已做**（2026-09-01 约 22 s）：显示 FrameTimeline 2155/2155 On-time + Valid prediction；预测器 recovery 关着；SurfaceView layer timeline n/a。  
5. **D22** — **已做**（2026-09-01 系统设置锁，不改 app 投票）：本约 60 s 25 ms 回退类消失；stuffing 钉在约 29 ms。  
6. **D3 时间戳** — **已做**（2026-09-01）：`ANativeWindow_setBuffersTimestamp` 默认 2 拍；`desired2present` 29 ms → 5 ms，仍 125 fps。  
7. **N2** — 仅当 Dawn C 回收时 UAF / SIGSEGV（崩溃车道）。  
8. **D13 / keep vs BLAST** — **Closed（加重）：**keep-6 提高抖动频率；BLAST 仍是 5。已退回 keep-3。不要再叠 keep。

## 4. 本路径已保持

- H1 / C2 / H8 / Fifo / intern 一条 queue / keep-3（ND-SURF；keep-6 已撤回）。  
- 不抄 C7 AAR 泄漏 batch。  
- `preferred_canvas_format` 复用 guest device（N6）。  
- 仓外 host：峰值模式 + 同 Hz `setFrameRate`（H24 / H27）。不要再加投票。

## 5. D25 之后的瓶颈（hitch 分支探针）

不是毫秒热点。hitch 分支 JNI 路径工作量是 **约 7.1 ms / 8.33 ms 拍**。两级：

1. **D2（链路中段）— 相位源。** CM 泵把 guest 串进 present deadline。`wakeA` condvar 稳定（0.5–1.3 ms）。15 次 androidx JNI import（约 3.3 ms）**不是**这次弹出的必要条件（D25）。**Dawn C 2026-09-01：** `vsync→wgpuSurfacePresent` 在 2880 次 present 上 `>8.3ms=0`（JNI 路径约 19%）。延迟仍在拍内游荡 0.8–6.9 ms。剩下的抖动不必跨过 vsync 边界。**P2/P3（95 s / 12000 帧，2026-09-01）：**present `phase-crossing` 2（间隔 55.5 s，极慢漂移）；retire 存活 `<8.3ms=0`。**P4（95 s / 12720 帧，2026-09-01）：**guest 转角时钟 `angleDt 8-9ms=全部`、`9-17ms=0`、`>17ms=0`——**变换矩阵时间严格线性**。**P5（约 12 min，2026-09-02）：**`take-skip` **0 次**——guest 的 `now`/`angle` 每拍 8.33 ms 步进，**与 present 相位解耦**。**相位补充（2026-09-02）：**重读完整 `phase-crossing` 序列发现 present 相位会**偶发失锁（snap）**——23:44 一次 `lat` 5→9.6 ms、`margin` +3→−1.3 ms 一拍内翻负、此后 `cross` 每秒 +2~5；但 30 分钟仅见一次，且 P5 证实 snap **不传递到 guest 内容**（take-skip 仍 0）。CM 泵 / `GfxOnFrameGate` 锁相与 host present/buffer 生命周期、guest 内容三条**都干净**——出阵路径不是抖动根因。
2. **D3/D19（链路末端）— 眼睛看见的地方。** 无时间戳的 `present` + **5** 张 BLAST 把队列塞到约 29 ms（D22 锁下）。**时间戳 2 拍**（2026-09-01）：`wgpuSurfacePresent` 前 `ANativeWindow_setBuffersTimestamp`；`desired2present` **5 ms=7072**，FPS 125。显示 FrameTimeline 仍是 On-time。若还有画面回退，D13 是症状。**D20 Closed（2026-09-01）：**SF 无降帧。**N9（2026-09-01）：**present 提交路径本身是干净的 — configure 参数（format `caps.formats[0]`、`Fifo`、`alpha=caps.alphaModes[0]`）、Dawn commit（同 androidx AAR SHA）、以及 65 s SF 窗口（`droppedFrames=0`、`jankyFrames=0`、`present2presentDelta 25ms=0`、`desired2present 5ms=7811`）在 Dawn C 路径上**无 rewind/drop/jank**。

**结论（2026-09-02，暂停验证）：** 三侧可观测层——guest 内容（P4/P5 线性、与相位解耦）、SF 计数器（`droppedFrames=0`、`jankyFrames=0`、无 rewind/drop）、present 提交（N9 干净）——**全部干净**。视觉「弹」因此落在**未被计量的层**：最可能是 SF 计数器之外的合成器/面板 BLAST 重播（D13/D19 的 rewind），或偶发 present-phase snap 的 host 侧瞬态——两者都**尚未在弹的瞬间被逐帧抓帧证实**。下一步需事件触发式抓帧（screenrecord 与 `sinceLast` 骤降联动，精确抽取 snap/弹 前后帧判定方向），或补真实 `onSubmittedWorkDone` fence 关闭 D24 最后一个结构差异。

修复排序：present 时间戳 **已落地**。剩下 draw-present 解耦。不要再砍 JNI。本探针立方体 host：仓外 `hosts/fullscreen-surface` + `GpuBackends.dawn()` + `Store.bindCanvasNativeWindow`。

## 6. 重开：跟踪热路径（issue 300）

Agent 队列：[`../agent/gfx-hitch.zh.md`](../agent/gfx-hitch.zh.md)。下一刀：`python3 ./scripts/gfx-hitch-remaining.py`。此前 Closed / Likely / Mitigated 行只作**档案**。这次重开**不把它们当前提**。一刀一个变量。不要给上游提 issue。

要对上的症状（眼睛，不是计数器）：NativeGpu / Dawn C 旋转立方体在 Vivo V2458A（Android 16，设置锁 120 Hz）上仍会**肉眼弹出**（约 5 s 一档）。对照：`hosts/native-webgpu` androidx 立方体（D24）**不弹**。

### 6.1 一拍序列（代码，不是结论）

```text
UI Choreographer.doFrame(frameTimeNanos)
  → Store.postGfxVsync → JNI nativeStorePostGfxVsync
  → GfxOnFrameGate.post          // in_frame || pending 则丢（1 槽）

GpuThread / 8MiB wasmtime-cm-pump
  wasi-gfx surface.on-frame  poll_produce
    → wait_take                  // condvar；1:1；锁 last_take_gen
    → note_take_vsync            // clocks.now / 立方体转角
    → 写 frame-event {nothing}

Guest（仓外 MoonBit 立方体）
  clocks.now → angle += ω·dt
  get-current-texture
  create-view / encoder / begin-render-pass / set-* / draw / end / finish
  write-buffer-with-copy         // mat4
  queue.submit
  context.present                // H8：自动 present 之后是空操作

NativeGpuHost（同一泵线程）
  canvas_current_texture:
    丢掉未 present 的
    wgpuInstanceProcessEvents
    wgpuSurfaceGetCurrentTexture
    pending_present = texture
  write_buffer_with_copy:
    wgpuQueueWriteBuffer         // 线性内存一拷
  queue_submit:
    wgpuQueueSubmit
    canvas_present:              // H8 自动
      ANativeWindow_setBuffersTimestamp（默认 2 拍）
      wgpuSurfacePresent
      presented_ring.push        // keep-3
    mark_canvas_gpu_done()       // 整环立刻标完；没有 wgpuQueueOnSubmittedWorkDone
    retire + wgpuTextureRelease

SurfaceFlinger / BLAST / 面板
```

### 6.2 这次重开计量什么

`GfxHitch` 的 `hotpath`（每 120 次 present）和 `hotpath-spike`（任一阶段超阈值）。`Instant` 分段耗时，不是「已关闭 ID」的直方图。

| 阶段 | 代码 | 冒尖意味着 |
|------|------|------------|
| S3a events | acquire 上的 `wgpuInstanceProcessEvents` | Dawn 事件泵 |
| S3b acquire | `wgpuSurfaceGetCurrentTexture` | BLAST / Dawn 阻塞 |
| S0→S3 | Choreographer 时间戳 → acquire 起点 | 门 / CM 唤醒 + guest 开工 |
| S5 write | `wgpuQueueWriteBuffer` | 拷贝 / 队列卡住 |
| S4 encode gap | acquire 之后 → submit 起点 | guest WIT + 编码 |
| S6a submit | `wgpuQueueSubmit` | GPU 队列 |
| S6b present | 盖戳 + `wgpuSurfacePresent` | 合成器队列 |
| S6c retire | `mark_canvas_gpu_done` + keep-3 释放 | CPU 回收 |
| S0→S6 | 已有 `lastLatencyNs` | 进程内整拍 |

### 6.3 砍刀顺序（先有数，再动一刀）

0. 在 V2458A、设置锁 120 Hz 上收 ≥2 min 的 `hotpath` + `hotpath-spike`。把肉眼弹出绑到某个阶段冒尖，**或**绑到「弹出时进程内无冒尖」。
1. S3 / S6b 冒尖 → 合成器 / acquire（BLAST、时间戳、Fifo）。只动一个旋钮。
2. S4 encode-gap 冒尖 → guest / CM（不是 Dawn C）。只动一个旋钮。
3. S6a 冒尖，或 CPU 不冒尖但眼睛在弹 → GPU / fence 寿命 vs D24 的 `onSubmittedWorkDone`。今天的 `dawn_c.rs` **没有**绑这个符号。
4. 弹出时进程内无冒尖 → 进程外的合成器 / 面板。用 `hotpath-spike` 或 `phase-crossing` 做事件触发 `screenrecord`。

在某个阶段认领这次弹出之前，**不要**再叠 keep / DisplayManager / GameState / 砍 JNI。

### 6.4 相对 D24 的结构差异（代码事实）

- D24 立方体：没有 Wasmtime；androidx `GPUSurface.present`；每 2 次 present 一次真 `onSubmittedWorkDone`（C7 batch）。
- NativeGpu 立方体：Wasmtime CM 泵 + `GfxOnFrameGate`；`queue.submit` 自动 present（H8）；`mark_canvas_gpu_done` 是立刻标完。

### 6.5 云环境 vs 真机

云环境**模拟不了**真机 present 路径。没有 Android，没有 `ANativeWindow`，没有 Mali/Vulkan Dawn `.so`，没有 BLAST / SurfaceFlinger / 120 Hz 面板。Linux 上 `hitch_monotonic_ns` 为 0，所以 `present n=` / `phase-crossing` / retire 存活直方图在这里不跑。`try_load_dawn_c` 尽力而为，slot 仍是 0。

云环境**能**查的是进程内节拍机：用合成的 Choreographer 时间戳跑 `GfxOnFrameGate` 1:1 + `NativeGpuHost` acquire → write → submit → H8 空操作 → keep-3 → desired-present 节奏 → `vsync_dt` 桶。这就是 `hotpath_synthetic_120hz_beats_are_1_to_1`。云上绿**关不掉**肉眼弹出。

### 6.6 真机窗口（HP-LOG）

2026-09-02 采于 V2458A（PD2415），设置 `min_refresh_rate=120` / `peak_refresh_rate=120`。Host：仓外 `hosts/fullscreen-surface` `installDebug`（对本仓 arm64 `libwasmtime_android_kt.so` 在热路径探针之后重编）。Guest：旋转立方体。实时 logcat `GfxHitch` + `FullscreenSurface` **150 s**（09:24:00.992–09:26:31.127）。`present n` 33240 → **51240**（18000 次 present）。不要用档案 Closed 下结论。肉眼弹出：**本窗口未确认**（无观察者）；进程一直在出帧。

Choreographer（累计到 n=51240）：`<11ms=51240` `11-20ms=0` `>20ms=0` `lastDtNs≈8.31ms` `dispHz=120.00001` `modeId=1`。

`hotpath` 窗口：**151**（每 120 次 present）。窗口 **last** Instant（151 个 last 的中位 / 最小 / 最大）：

| 阶段 | 中位 | 最小 | 最大 |
|------|------|------|------|
| S3a events | 143 µs | 22 µs | 354 µs |
| S3b acquire | 489 µs | 121 µs | 7.60 ms |
| S5 write | 49 µs | 19 µs | 490 µs |
| S4 encode-gap | 348 µs | 178 µs | 845 µs |
| S6a submit | 425 µs | 147 µs | 1.45 ms |
| S6b present | 1.35 ms | 334 µs | 3.97 ms |
| S6c retire | 231 ns | 0 | 539 ns |

各窗口 **max** 再取最大：events 1.61 ms，acquire **8.36 ms**，write 1.08 ms，encode-gap **17.9 ms**，submit 1.57 ms，present 3.97 ms，retire 0.19 ms。encode-gap 最大只出现在一个窗口（`hotpath n=45960`）。

本段 `hotpath-spike`：**6289** 行。超阈值（2 ms；encode 6 ms）：**acquire 5433**，present 916，encode-gap 1，events/write/submit/retire 0。acquire>2 ms 是一段**连续**突发 09:25:46–09:26:31（**45.1 s**，间隔 4–12 ms ≈ 每拍），不是孤立的约 5 s 空隙。45/151 个窗口 last-acquire >2 ms；21/151 last-present >2 ms。

n=51240 累计直方图（含本段之前）：acquire 间隔 `<11ms=51207` `11-20ms=33` `>20ms=0`。present `lastLatencyNs` `<8ms=46243` `8-16ms=4997` `>16ms=1` `>8.3ms=4760`；间隔 `<11ms=51070` `11-20ms=169` `>20ms=1`；`cross=347`；retire 存活 `<8.3ms=0` `8.3-25ms=27045` `>25ms=24192`；`angleDt` `8-9ms=51239` `9-17ms=1`。

**绑定：** 这 150 s **没有约 5 s 周期的进程内阶段冒尖**；成簇超阈值的只有一段 45 s、每拍一次的 **S3b acquire >2 ms**（S6b present >2 ms 频繁但无周期）。肉眼弹出**未确认**，因此本窗口若有约 5 s 弹出，就是 **弹出时没有孤立的进程内冒尖** —— 具名后续是合成器/面板 / 有人盯着的事件 `screenrecord`，不是 S4/S6a。
