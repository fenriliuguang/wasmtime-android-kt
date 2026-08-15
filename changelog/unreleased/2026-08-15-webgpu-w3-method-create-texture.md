### Code — wasi:webgpu W3 [method]gpu-device.create-texture (2026-08-15)

- Register `[method]gpu-device.create-texture` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed 1×1 RGBA8 RENDER_ATTACHMENT, still u32)
- Fixture `fixtures/w1/webgpu_method_create_texture`; native `wasi_webgpu_method_create_texture`; twin instrument `WasiWebGpuMethodCreateTextureInstrumentedTest`
- Transitional: no Guest `gpu-texture-descriptor`; still u32, not `gpu-texture` resource. Not compliance
