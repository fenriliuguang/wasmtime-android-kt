### Code — wasi:webgpu W3 [method]gpu-device.create-sampler (2026-08-15)

- Register `[method]gpu-device.create-sampler` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed default sampler, still u32)
- Fixture `fixtures/w1/webgpu_method_create_sampler`; native `wasi_webgpu_method_create_sampler`; twin instrument `WasiWebGpuMethodCreateSamplerInstrumentedTest`
- Transitional: no Guest `option<gpu-sampler-descriptor>`; still u32, not `gpu-sampler` resource. Not compliance
