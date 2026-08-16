### Code — wasi:webgpu W3 [method]gpu-device.create-compute-pipeline (2026-08-16)

- Register `[method]gpu-device.create-compute-pipeline` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed stub WGSL + empty pipeline-layout, still u32)
- Fixture `fixtures/w1/webgpu_method_create_compute_pipeline`; native `wasi_webgpu_method_create_compute_pipeline`; twin instrument `WasiWebGpuMethodCreateComputePipelineInstrumentedTest`
- Transitional: no Guest `gpu-compute-pipeline-descriptor`; still u32, not `gpu-compute-pipeline` resource; Cpu 要求显式 layout. Not compliance
