### Code — L2 canvas present host-owned window (2026-08-21)

- Host-owned Android window behind product `[method]gpu-canvas-context.configure` / `get-current-texture`; Dawn creates a GPUSurface internally (no product `surface-*` guest names)
- Instrument `WasiWebGpuMethodCanvasContextPresentInstrumentedTest` binds the window on GpuThread, guest returns harness `1`, host clears + presents; no CTS / compliance claim
- Fixture `webgpu_method_canvas_context_present`; native module of the same name