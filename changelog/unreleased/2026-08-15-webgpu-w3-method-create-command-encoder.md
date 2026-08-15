### Code — wasi:webgpu W3 [method]gpu-device.create-command-encoder (2026-08-15)

- Register `[method]gpu-device.create-command-encoder` on existing `gpu-device` / `get-device` (sync; L2 `request-adapter` then `adapter-request-device` then `device-create-command-encoder`, same u32 as the flat name)
- Fixture `fixtures/w1/webgpu_method_create_command_encoder`; native `wasi_webgpu_method_create_command_encoder`; twin instrument `WasiWebGpuMethodCreateCommandEncoderInstrumentedTest`
- Transitional: no `option<descriptor>`; still u32. Flat `device-create-command-encoder` kept. Not compliance
