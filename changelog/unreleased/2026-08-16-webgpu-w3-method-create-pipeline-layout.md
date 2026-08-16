### Code — wasi:webgpu W3 [method]gpu-device.create-pipeline-layout (2026-08-16)

- Register `[method]gpu-device.create-pipeline-layout` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed empty bind-group-layouts, still u32)
- Fixture `fixtures/w1/webgpu_method_create_pipeline_layout`; native `wasi_webgpu_method_create_pipeline_layout`; twin instrument `WasiWebGpuMethodCreatePipelineLayoutInstrumentedTest`
- Transitional: no Guest `option<gpu-pipeline-layout-descriptor>`; still u32, not `gpu-pipeline-layout` resource. Not compliance
