### Code — wasi:webgpu W3 [method]gpu-device.create-render-pipeline (2026-08-16)

- Register `[method]gpu-device.create-render-pipeline` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed stub WGSL + triangle pipeline RGBA8, still u32)
- Fixture `fixtures/w1/webgpu_method_create_render_pipeline`; native `wasi_webgpu_method_create_render_pipeline`; twin instrument `WasiWebGpuMethodCreateRenderPipelineInstrumentedTest`
- Transitional: no Guest `gpu-render-pipeline-descriptor`; still u32, not `gpu-render-pipeline` resource. Not compliance
