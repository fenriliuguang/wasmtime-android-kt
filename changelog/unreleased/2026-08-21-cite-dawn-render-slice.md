### Code — cite Dawn render slice (2026-08-21)

- Canonical `[method]` guest chains create-buffer → create-texture → create-view → command-encoder → begin-render-pass → end → finish → `gpu-queue.submit` on `DawnWasiWebGpuHost` (not Cpu)
- Instrument `WasiWebGpuDawnRenderSliceInstrumentedTest` runs the slice on one GpuThread; no CTS / compliance claim
- Fixture `webgpu_method_dawn_render_slice`; native module of the same name
