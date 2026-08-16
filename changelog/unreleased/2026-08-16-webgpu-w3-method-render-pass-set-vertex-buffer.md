### Code — wasi:webgpu W3 [method]gpu-render-pass-encoder.set-vertex-buffer (2026-08-16)

- Register `[method]gpu-render-pass-encoder.set-vertex-buffer` on existing `gpu-render-pass-encoder` / `get-pass` (sync void; L2 adapter → device → encoder → begin-render-pass-clear with Cpu offscreen view then host-fixed VERTEX buffer at slot 0; Guest stub buffer ignored)
- Fixture `fixtures/w1/webgpu_method_render_pass_set_vertex_buffer`; native `wasi_webgpu_method_render_pass_set_vertex_buffer`; twin instrument `WasiWebGpuMethodRenderPassSetVertexBufferInstrumentedTest`
- Transitional: Guest returns stub 31. Not compliance
