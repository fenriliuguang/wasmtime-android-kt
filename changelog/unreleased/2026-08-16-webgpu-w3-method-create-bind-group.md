### Code — wasi:webgpu W3 [method]gpu-device.create-bind-group (2026-08-16)

- Register `[method]gpu-device.create-bind-group` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed empty BGL + empty entries, still u32)
- Fixture `fixtures/w1/webgpu_method_create_bind_group`; native `wasi_webgpu_method_create_bind_group`; twin instrument `WasiWebGpuMethodCreateBindGroupInstrumentedTest`
- Transitional: no Guest `gpu-bind-group-descriptor`; still u32, not `gpu-bind-group` resource. Not compliance
