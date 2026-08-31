# 帧循环实现建议

[English](frame-loop-suggestion.md) | **中文**

**地位：gfx RFC 的形状笔记。** `0.1.0` 帧循环已落地。现行 host 消费：[`../agent/native-dawn.md`](../agent/native-dawn.md)。不要改 guest 拉流形状。与英文冲突时以英文为准。

本页记录 **已接受** 的 `0.1.0` gfx 连续上屏草图。**不**加入 [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md)。**不**重开 P0。**NG-9 仍禁止把 gfx 升为 P0**；产品门禁是 [gfx RFC](../scheme/rfc-wasi-gfx-frame-loop.md)，不是本页。

`0.1.0` 循环已落地。native Dawn present 接到同一 `GfxOnFrameGate`（**ND-SURF**）。今日上屏是单帧 WG-6 以及 vsync 节拍的 `WasiGfxFrameLoopInstrumentedTest`。

上游草图对齐 [wasi-gfx/wasi-gfx](https://github.com/wasi-gfx/wasi-gfx) 标签 **`v0.2.0`**（`wasi-gfx:surface@0.2.0` + `surface-webgpu` 引用 `wasi:webgpu@0.3.0-rc.2`）。已 vendor：[`../../third_party/wasi-gfx/v0.2.0/wit/surface.wit`](../../third_party/wasi-gfx/v0.2.0/wit/surface.wit)。

## 1. 不要 JS 式 scheduler 包

`wasi:webgpu` 没有 `present` / vsync / rAF。显示面靠 gfx，不要塞进 webgpu。

不要 `start: func(callback: func(ts: u64) -> bool)`：和 WASI 0.3 的 `stream` / `async func` 冲突，会从 Choreographer 重入 guest，并撞上「每个 Store 一个 `run_concurrent` 驱动方」。

gfx 已有形状：guest **拉** `on-frame` stream；host 在 **GpuThread** 上把 vsync 写成流事件。

## 2. 架构

```text
Guest     async command.run
          loop: 读 on-frame → get-current-texture → encode → submit → present
WIT       surface.on-frame stream          节拍
          surface-webgpu.context           交换链胶水 + present
          wasi:webgpu                      GPU；无 rAF
Host      run_concurrent  单 Store 单驱动
          GpuThread       Dawn / present / 写 stream
          UI              Surface 生命周期 + Choreographer 投递；不碰 Dawn
```

一帧：`doFrame` → 投递 GpuThread → guest 未消费则 **丢拍** → 否则写 `frame-event` → guest 读 → `get-current-texture` → `submit` → `present`（Fifo 不要再等一次 vsync）。

停止：guest drop stream/`surface`；或 `surfaceDestroyed` 时在 GpuThread **关闭 stream**，再 `unconfigure`。

## 3. 建议 WIT（示意）

在 gfx 上补 TODO，不要平行的 `wasi:frame-scheduler`。`on-frame`：每 surface 一条流；慢 guest 丢旧拍；`frame-event` 应对齐 `wasi:clocks` monotonic instant。`present` 在本仓 Dawn 路径保持显式。完整 WIT 见英文 §3。

```wit
package example:triangle@0.1.0;

world windowed-webgpu {
  include wasi:cli/command@0.3.0;
  import wasi-gfx:surface/surface@0.2.0;
  import wasi-gfx:surface/surface-webgpu;
  import wasi:webgpu/webgpu@0.3.0-rc.2;
}
```

`run` 必须 **async**。Host `on-frame`（P010-GFXH）返回 CM `stream<frame-event>`；vsync 载荷在名为 `GpuThread` 的 helper 线程上产生。钉里 `on-frame` 是同步 `func`（不是 `async func`）；本仓未开 Wasmtime stackful CM async。present / `surface-webgpu` 已 P010-GFXL（两帧预缓冲，非 vsync 节拍）。立方体抖动排查：[`gfx-hitch-checklist.zh.md`](gfx-hitch-checklist.zh.md)。

## 4. MoonBit guest（示意）

MoonBit `async fn` **没有** `await` 关键字。`@surface` / `read` 为占位，须从 WIT 生成。本仓不编译此代码。

```moonbit
/// Illustrative guest for world windowed-webgpu.
/// Not compiled here. Replace @surface / @webgpu with wit-bindgen output.

async fn run() -> Result[Unit, String] {
  let s = @surface.Surface::new({
    width: Some(720),
    height: Some(1280),
  })
  let gpu = @webgpu.Gpu::get()
  let adapter = match gpu.request_adapter(None) {
    Some(a) => a
    None => return Err("no adapter")
  }
  let device = match adapter.request_device(None) {
    Ok(d) => d
    Err(e) => return Err(e.message)
  }
  let queue = device.queue()
  let format = gpu.get_preferred_canvas_format()

  let ctx = @surface_webgpu.Context::new(s)
  ctx.configure({
    device: device,
    format: format,
    usage: Some(@webgpu.GpuTextureUsage::RenderAttachment),
    view_formats: None,
    color_space: None,
    tone_mapping: None,
    alpha_mode: None,
  })

  // One stream per surface. Do not call on_frame a second time.
  let frames = s.on_frame()
  loop {
    match frames.read() {
      // Stream closed: Surface gone or host stopped ticks.
      None => break
      Some(_ev) => {
        let tex = ctx.get_current_texture()
        let encoder = device.create_command_encoder(None)
        // begin_render_pass(tex) → draw(3) → end → finish
        let commands = encoder.finish(None)
        queue.submit([commands])
        ctx.present()
      }
    }
  }
  Ok(())
}
```

## 5. Kotlin 接线（示意）

尚不在 `:host-dawn`。节拍只留一个：Choreographer **或** 阻塞 Fifo `present`。

```kotlin
private val frameCallback = object : Choreographer.FrameCallback {
    override fun doFrame(frameTimeNanos: Long) {
        if (!running.get()) return
        gpuHandler.post { onVsync(frameTimeNanos) }
        Choreographer.getInstance().postFrameCallback(this)
    }
}

fun onSurfaceDestroyed() {
    running.set(false)
    Choreographer.getInstance().removeFrameCallback(frameCallback)
    gpuHandler.post {
        frameStream.close()
        host.canvasContextUnconfigure(contextId)
        host.unbindCanvasNativeWindow()
    }
}

private var pendingUnconsumed = false

fun onVsync(frameTimeNanos: Long) {
    if (!surfaceAlive) return
    if (pendingUnconsumed) {
        return // 丢拍，不排队
    }
    pendingUnconsumed = true
    frameStream.write(FrameEvent(instantNanos = frameTimeNanos))
}

fun presentAfterSubmit() {
    gpuSurface.present()
}
```

GpuThread 上一次 `callRunConcurrent`：**阻塞到 guest `run` 返回**。循环在 guest 内。拆 Surface 靠关 stream 解开 `run`。JNI 写 stream 须弹回 Java 调用方（8MiB CM 泵约束）。

## 6. 若将来升格 RFC 仍缺什么

Host `on-frame` 已 P010-GFXH。产品骨架循环已 P010-GFXL。产品 adapter/device 已 **P010-GFXB**。Choreographer vsync + 关 stream 已 **P010-GFXV**。**`0.1.0` 剩余自动刀：** 仓外 demo README 链接 + 真机行（**P010-DEMO**）。不要把本页当成 P2 `Next:`。
