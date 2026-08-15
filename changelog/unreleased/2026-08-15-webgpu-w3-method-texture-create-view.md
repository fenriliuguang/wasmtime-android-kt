### Code — wasi:webgpu W3 [method]gpu-texture.create-view (2026-08-15)

- Register WIT `gpu-texture` + `get-texture` + `[method]gpu-texture.create-view` (sync; L2 adapter → device → host-fixed 1×1 texture → create-view, still u32)
- Fixture `fixtures/w1/webgpu_method_texture_create_view`; native `wasi_webgpu_method_texture_create_view`; twin instrument `WasiWebGpuMethodTextureCreateViewInstrumentedTest`
- Transitional: no Guest `option<gpu-texture-view-descriptor>`; still u32, not `gpu-texture-view` resource. Not compliance
