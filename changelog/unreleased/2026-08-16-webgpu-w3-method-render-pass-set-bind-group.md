### Code — wasi:webgpu W3 [method]gpu-render-pass-encoder.set-bind-group (2026-08-16)

- Register `[method]gpu-render-pass-encoder.set-bind-group` on existing `gpu-render-pass-encoder` / `get-pass` (sync void; L2 adapter → device → encoder → begin-render-pass-clear with Cpu offscreen view then host-fixed empty bind-group at index 0; Guest stub bind-group ignored)
- Fixture `fixtures/w1/webgpu_method_render_pass_set_bind_group`; native `wasi_webgpu_method_render_pass_set_bind_group`; twin instrument `WasiWebGpuMethodRenderPassSetBindGroupInstrumentedTest`
- Transitional: Guest returns stub 67. Not compliance
