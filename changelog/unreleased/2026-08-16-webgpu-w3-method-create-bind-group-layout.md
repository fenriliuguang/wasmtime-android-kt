### Code — wasi:webgpu W3 [method]gpu-device.create-bind-group-layout (2026-08-16)

- Register `[method]gpu-device.create-bind-group-layout` on existing `gpu-device` / `get-device` (sync; L2 adapter → device then host-fixed empty entries, still u32)
- Fixture `fixtures/w1/webgpu_method_create_bind_group_layout`; native `wasi_webgpu_method_create_bind_group_layout`; twin instrument `WasiWebGpuMethodCreateBindGroupLayoutInstrumentedTest`
- Transitional: no Guest `gpu-bind-group-layout-descriptor` / entries; still u32, not `gpu-bind-group-layout` resource. Not compliance
