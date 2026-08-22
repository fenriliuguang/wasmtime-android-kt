# WG-6 guest-drawn canvas present

- `DawnWasiWebGpuHost`: `get-preferred-canvas-format` returns Android native `bgra8unorm` when a window is bound; fragment targets follow the configured swapchain format. Guest `create-view` none uses Dawn `createView()` defaults; present once after `queue.submit`.
- Fixture `webgpu_method_dawn_guest_canvas_present`: guest queries preferred format (JS-like), then `configure` + `get-current-texture` + vertex `draw(3)` + submit.
- `ExperimentalWebGpuBridge.attachDawnGuestCanvasPresent`; instrument `WasiWebGpuDawnGuestCanvasPresentInstrumentedTest`; `examples/webgpu-guest-canvas-present/README.md`.
