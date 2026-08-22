# WG-6 guest-drawn canvas present

- `DawnWasiWebGpuHost`: `get-current-texture` acquires the swapchain (no host clear); present after guest `queue.submit` (or swapchain texture drop). Surface configure prefers the guest `format` when the adapter lists it.
- Fixture `webgpu_method_dawn_guest_canvas_present`: guest `configure` + `get-current-texture` + vertex `draw(3)` + submit (not create-texture 1×1, not host-clear cite).
- `ExperimentalWebGpuBridge.attachDawnGuestCanvasPresent`; instrument `WasiWebGpuDawnGuestCanvasPresentInstrumentedTest`; `examples/webgpu-guest-canvas-present/README.md`.
