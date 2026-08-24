# 帧循环实现建议

[English](frame-loop-suggestion.md) | **中文**

**地位：仅建议 — 不是计划、不是切刀、不是 DoD。** 与英文冲突时以英文为准。

本页记录：若本仓以后做连续上屏，host/guest 可以长什么样。**不**加入 [`../agent/wasi-p3.md`](../agent/wasi-p3.md) / [`../scheme/wasi-p3-surface.md`](../scheme/wasi-p3-surface.md)。**不**重开 P0 wasi:webgpu。**不**把 `wasi-gfx` 升为近端工作（NG-9 / DG-6 仍要单独 RFC）。

现行队列仍是 P1 WASI 0.3。今日上屏是单帧 WG-6。线程：[`threading-android.md`](threading-android.md)。

上游草图对齐 [wasi-gfx/wasi-gfx](https://github.com/wasi-gfx/wasi-gfx) `packages/surface`（2026-08：`wasi-gfx:surface@0.2.0` + `surface-webgpu` 引用 `wasi:webgpu@0.3.0-rc.2`）。本仓 **未 vendor** 该树。

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

`run` 必须 **async**。未托管 gfx `surface` 之前，host 可用同构的 `request-frame: async func() -> timestamp` 作站位，不要当产品名。

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

托管 `wasi-gfx:surface`；`on-frame` 背压（与 P1 W1 相关，但 W1 不是本循环）；可取消的 `run`；多帧 swapchain；每帧不要在泵上 latch；MoonBit/Rust 绑定。在此之前保持单帧 WG-6，不要把本页当成 `wasi-p3-remaining` 的 Next。
