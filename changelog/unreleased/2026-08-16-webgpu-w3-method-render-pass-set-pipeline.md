### Code — wasi:webgpu W3 [method]gpu-render-pass-encoder.set-pipeline (2026-08-16)

- Register `[method]gpu-render-pass-encoder.set-pipeline` on existing `gpu-render-pass-encoder` / `get-pass` (sync void; L2 adapter → device → encoder → begin-render-pass-clear with Cpu offscreen view then host-fixed triangle pipeline; Guest stub pipeline ignored)
- Fixture `fixtures/w1/webgpu_method_render_pass_set_pipeline`; native `wasi_webgpu_method_render_pass_set_pipeline`; twin instrument `WasiWebGpuMethodRenderPassSetPipelineInstrumentedTest`
- Transitional: Guest returns stub 71. Not compliance
