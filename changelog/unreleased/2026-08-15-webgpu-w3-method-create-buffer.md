### Code — wasi:webgpu W3 [method]gpu-device.create-buffer (2026-08-15)

- Register `[method]gpu-device.create-buffer` on existing `gpu-device` / `get-device` (sync; L2 `request-adapter` then `adapter-request-device` then host-fixed `device-create-buffer`, still u32)
- Fixture `fixtures/w1/webgpu_method_create_buffer`; native `wasi_webgpu_method_create_buffer`; twin instrument `WasiWebGpuMethodCreateBufferInstrumentedTest`
- Transitional: no Guest `gpu-buffer-descriptor`; still u32, not `gpu-buffer` resource. Not compliance
