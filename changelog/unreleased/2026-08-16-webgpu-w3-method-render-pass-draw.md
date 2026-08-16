### Code — wasi:webgpu W3 [method]gpu-render-pass-encoder.draw (2026-08-16)

- Register `[method]gpu-render-pass-encoder.draw` on existing `gpu-render-pass-encoder` / `get-pass` (sync void; L2 adapter → device → encoder → begin-render-pass-clear with Cpu offscreen view then host-fixed triangle set-pipeline + draw(3); Guest vertexCount ignored)
- Fixture `fixtures/w1/webgpu_method_render_pass_draw`; native `wasi_webgpu_method_render_pass_draw`; twin instrument `WasiWebGpuMethodRenderPassDrawInstrumentedTest`
- Transitional: Guest returns stub 29. Not compliance
