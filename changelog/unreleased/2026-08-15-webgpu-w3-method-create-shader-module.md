### Code — wasi:webgpu W3 [method]gpu-device.create-shader-module (2026-08-15)

- Register `[method]gpu-device.create-shader-module` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed stub WGSL, still u32)
- Fixture `fixtures/w1/webgpu_method_create_shader_module`; native `wasi_webgpu_method_create_shader_module`; twin instrument `WasiWebGpuMethodCreateShaderModuleInstrumentedTest`
- Transitional: no Guest `gpu-shader-module-descriptor` / string; still u32, not `gpu-shader-module` resource. Not compliance
