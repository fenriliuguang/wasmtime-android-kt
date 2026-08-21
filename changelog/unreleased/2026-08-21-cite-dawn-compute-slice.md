### Code — cite Dawn compute slice (2026-08-21)

- Canonical `[method]` guest chains create-buffer → command-encoder → begin-compute-pass → end → finish → `gpu-queue.submit` on `DawnWasiWebGpuHost` (not Cpu)
- Instrument `WasiWebGpuDawnComputeSliceInstrumentedTest` runs the slice on one GpuThread; no CTS / compliance claim
- Fixture `webgpu_method_dawn_compute_slice`; native module of the same name
