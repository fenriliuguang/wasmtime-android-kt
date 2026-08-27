# Frame-loop implementation suggestion

**English** | [中文](frame-loop-suggestion.zh.md)

**Status: shape notes for [`rfc-wasi-gfx-frame-loop.md`](../scheme/rfc-wasi-gfx-frame-loop.md). Remaining gfx auto cut: [`product-010.md`](../agent/product-010.md) `P010-DEMO`. Last `0.1.0` auto cut: `P010-DEMO` (README, not this file).**

This page records host/guest sketches for the **accepted** `0.1.0` gfx present loop. It does **not** add a lane to [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md). It does **not** reopen P0 wasi:webgpu (G1–G9 / WG-6). **NG-9 still forbids promoting gfx to P0**; the product gate is the gfx RFC, not this file.

Living auto queue is **`0.1.0` complete frame loop** then demo: product adapter/device (**P010-GFXB**) then Choreographer vsync (**P010-GFXV**, landed) then **P010-DEMO**. On-screen today is one-shot WG-6 plus vsync-paced `WasiGfxFrameLoopInstrumentedTest`. Thread rules: [`threading-android.md`](threading-android.md). WIT pin is **P010-GFXP** (`v0.2.0`); host `on-frame` is P010-GFXH; skeleton present loop is P010-GFXL; vsync 1-slot is P010-GFXV.

Upstream sketches below follow [wasi-gfx/wasi-gfx](https://github.com/wasi-gfx/wasi-gfx) tag **`v0.2.0`** (`wasi-gfx:surface@0.2.0` + `surface-webgpu` importing `wasi:webgpu@0.3.0-rc.2`). Vendored [`../../third_party/wasi-gfx/v0.2.0/wit/surface.wit`](../../third_party/wasi-gfx/v0.2.0/wit/surface.wit). Names may move; do not pick a second tag without a changelog.

## 1. Why not a JS-style scheduler package

`wasi:webgpu` has no `present`, vsync, or `requestAnimationFrame`. Display/windowing belongs next to gfx, not inside webgpu.

Do **not** add:

```wit
start: func(callback: func(ts: u64) -> bool);
```

Passing a guest `func` for the host to invoke later fights WASI 0.3 (`stream` / `async func`), re-enters the component from Choreographer, and collides with “one `run_concurrent` driver per Store”.

Prefer the shape gfx already sketched: guest **pulls** `on-frame` as a CM `stream`. Host writes vsync events into that stream on **GpuThread**.

## 2. Architecture

```text
Guest component
  async wasi:cli/command#run
  loop: read on-frame → get-current-texture → encode → submit → present
        ─────────────────────────────────────────────────────────────
WIT    wasi-gfx:surface          stream<frame-event>     (tick)
       wasi-gfx:surface-webgpu   context + present       (swapchain glue)
       wasi:webgpu@0.3.0-rc.2    GPU objects; no rAF
        ─────────────────────────────────────────────────────────────
Host   Wasmtime run_concurrent   one Store, one driver
       GpuThread                 Dawn, GPUSurface, stream write, present
       UI / Choreographer        Surface lifecycle + vsync post; no Dawn
```

One frame:

```text
Choreographer.doFrame
  → post GpuThread
  → if guest has not consumed the previous event: drop this beat (no queue)
  → else write frame-event on on-frame
  → guest read resumes
  → context.get-current-texture
  → queue.submit
  → context.present()     // explicit Dawn; Fifo: do not also wait a second vsync
```

Stop:

- Guest drops the stream or the `surface` → host stops writing.
- Android `surfaceDestroyed` → GpuThread **closes** the stream so `read` completes, then `unconfigure`. Do not leave `run` blocked forever.

## 3. Suggested WIT (illustrative)

Pin is in-tree (`third_party/wasi-gfx/v0.2.0/`). Align with that tag; fill TODOs in a gfx fork/PR rather than a parallel `wasi:frame-scheduler`.

### 3.1 Tick (`wasi-gfx:surface`)

Upstream already has `on-frame: func() -> stream<frame-event>`. Suggested contracts (today the event is a placeholder `nothing: bool`):

- One stream per `surface`. Calling `on-frame` twice: same stream or trap — pick one and document it.
- One event ≈ one display beat that may `get-current-texture`.
- Slow guest: **coalesce / drop** old beats (do not unbounded-queue).
- Fast guest: reader blocks (stream backpressure).
- `frame-event` should carry a timestamp aligned with `wasi:clocks` monotonic **instant**, not a second millisecond clock.

### 3.2 Swapchain glue (`surface-webgpu`)

Upstream sketch:

```wit
interface surface-webgpu {
  use wasi:webgpu/webgpu@0.3.0-rc.2.{
    gpu-device, gpu-texture, gpu-texture-format,
    gpu-texture-usage, predefined-color-space,
    gpu-canvas-tone-mapping, gpu-canvas-alpha-mode,
  };
  use surface.{surface};

  record context-configuration {
    device: borrow<gpu-device>,
    format: gpu-texture-format,
    usage: option<gpu-texture-usage>,
    view-formats: option<list<gpu-texture-format>>,
    color-space: option<predefined-color-space>,
    tone-mapping: option<gpu-canvas-tone-mapping>,
    alpha-mode: option<gpu-canvas-alpha-mode>,
  }

  resource context {
    constructor(surface: borrow<surface>);
    configure: func(configuration: context-configuration);
    unconfigure: func();
    get-current-texture: func() -> gpu-texture;
    // TODO upstream: consider if needed
    present: func();
  }
}
```

For this Android host, keep **explicit** `present` (Dawn `GPUSurface.present`). Document: without a new `frame-event` since last present, a second `get-current-texture` is illegal or returns the previous texture — pick one. Lost/Outdated should become a `result` rather than a bare texture when gfx is ready.

This repo’s WG-6 path uses product `gpu-canvas-context.*` plus host-owned `ANativeWindow`. A gfx `surface` is an extra import; do not pretend it exists on the current smoke world.

### 3.3 Example world (guest)

```wit
package example:triangle@0.1.0;

world windowed-webgpu {
  include wasi:cli/command@0.3.0; // export async run
  import wasi-gfx:surface/surface@0.2.0;
  import wasi-gfx:surface/surface-webgpu;
  import wasi:webgpu/webgpu@0.3.0-rc.2;
}
```

`run` must be **async** so reading `on-frame` can yield. Official CLI `run` result landed as P1 W5 (`result`); a demo world may still keep a tiny export.

Host `on-frame` (P010-GFXH) returns a CM `stream<frame-event>`. **P010-GFXV:** UI Choreographer posts vsync into a 1-slot gate; `poll_produce` writes on **GpuThread**; unconsumed beats drop; `surfaceDestroyed` closes the stream. Pin `on-frame` is a sync `func` (not `async func`); this repo does not enable Wasmtime stackful CM async. Product `surface-webgpu` `context.present` is P010-GFXL. Cube hitch (device): [`gfx-hitch-checklist.md`](gfx-hitch-checklist.md).

## 4. MoonBit guest (illustrative)

MoonBit `async fn` does not use an `await` keyword; the compiler treats async calls as async. Bindings names (`@surface`, stream `read`) are **placeholders** — generate them from WIT. Wasm component + CM async support in the MoonBit toolchain is still moving; this is the intended control flow, not a buildable crate in this repo.

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

Triangle WGSL can match the existing WG-6 fixture (vertex_index fullscreen triangle). Stop the loop by dropping `frames` / `s` or returning from `run`.

## 5. Kotlin host wiring (illustrative)

Not in `:host-dawn` today. Same thread contract as [`threading-android.md`](threading-android.md). Use **one** beat: Choreographer **or** blocking Fifo `present`, not both.

```kotlin
// UI thread: vsync + Surface lifecycle only. Never windowFromSurface / Dawn here.

private val frameCallback = object : Choreographer.FrameCallback {
    override fun doFrame(frameTimeNanos: Long) {
        if (!running.get()) return
        gpuHandler.post { onVsync(frameTimeNanos) }
        Choreographer.getInstance().postFrameCallback(this)
    }
}

fun startLoop() {
    running.set(true)
    Choreographer.getInstance().postFrameCallback(frameCallback)
}

fun onSurfaceDestroyed() {
    running.set(false)
    Choreographer.getInstance().removeFrameCallback(frameCallback)
    gpuHandler.post {
        frameStream.close() // guest frames.read() → None / drop
        host.canvasContextUnconfigure(contextId)
        host.unbindCanvasNativeWindow()
    }
}

// GpuThread (HandlerThread): Dawn, stream write, present.

private var pendingUnconsumed = false

fun onVsync(frameTimeNanos: Long) {
    if (!surfaceAlive) return
    if (pendingUnconsumed) {
        // Backpressure: drop this beat; do not queue timestamps.
        return
    }
    pendingUnconsumed = true
    frameStream.write(FrameEvent(instantNanos = frameTimeNanos))
    // Native: complete the CM stream item the guest is blocked on.
    // Must not AttachCurrentThread on the 8MiB CM-pump pthread (see Instance.callRunConcurrent).
}

fun onGuestConsumedFrame() {
    pendingUnconsumed = false
}

fun presentAfterSubmit() {
    // Called from the same GpuThread as queue.submit (DawnWasiWebGpuHost today
    // presents on submit for the WG-6 pending slot).
    gpuSurface.present()
}
```

Instantiate once on GpuThread: `Engine` → `Component.compile` → bind window → attach webgpu (+ future gfx surface) → `callRunConcurrent`. That call **blocks until `run` returns**; the loop lives **inside** guest `run`, not as N Kotlin `callRunConcurrent` invocations. Closing the stream is how Android teardown unblocks `run`.

JNI/stream write must bounce L2 work to the Java caller thread (existing 8MiB `wasmtime-cm-pump` constraint).

## 6. What the gfx RFC still leaves to an implementation PR

The gfx RFC accepted the **pull `on-frame` stream** shape. P010-GFXP/H/L landed the skeleton. Remaining **`0.1.0` auto** cuts:

| Gap | Lane |
|-----|------|
| Product `request-adapter` / `request-device` in the frame-loop guest; `Linker.create` | **P010-GFXB** landed |
| Choreographer vsync write + drop unconsumed beats; `surfaceDestroyed` closes the stream | **P010-GFXV** landed |
| README **Demo** link to one out-of-tree wasm→runtime→present repo + named device row | **P010-DEMO** (not this file) |
| Swapchain Lost/Outdated, resize re-`configure` | Named later (`0.x`); not GFXB/GFXV |
| Dawn latch on the hot path / MoonBit bindings | Named later |

Do not treat this file as `wasmtime-p2-remaining`. Do not JS-style `start(callback)`.
