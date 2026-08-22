# Guest canvas present (WG-6)

Guest records a render pass on `gpu-canvas-context.get-current-texture`. The host presents after `queue.submit` — not a host-only clear, and not a 1×1 `create-texture` cite.

## Fixture

- **Wasm:** `fixtures/w1/webgpu_method_dawn_guest_canvas_present.wasm`
- **Product WIT:** `gpu-canvas-context.configure` + `get-current-texture` + shader / VERTEX buffer / render pipeline / `draw(3)` / `queue.submit`
- Regenerate: `python scripts/merge_guest_canvas_present_wat.py` then `wasm-tools parse` / `validate --features=cm-async,component-model`

## Android smoke (device)

1. Connect a device with Vulkan/WebGPU.
2. From repo root:

```powershell
.\gradlew :smoke-app:connectedDebugAndroidTest `
  -Pandroid.testInstrumentationRunnerArguments.class=io.github.fenriliuguang.wasmtime.android.smoke.WasiWebGpuDawnGuestCanvasPresentInstrumentedTest
```

Expect harness `1` and a green triangle on `MainActivity`'s `demoSurface` (guest fragment shader).

## Host attach

- Bind the window on the same GpuThread before instantiate: `DawnWasiWebGpuHost.bindCanvasNativeWindow`
- `ExperimentalWebGpuBridge.attachDawnGuestCanvasPresent` (render chain + canvas context)
