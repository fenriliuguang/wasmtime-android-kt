### Code — wasi:webgpu W3 command-encoder-begin-render-pass-clear dual-register (2026-08-14)

- Dual-register transitional flat `wasi:webgpu/webgpu@0.3.0-rc.2#command-encoder-begin-render-pass-clear` (sync u32; same L2 as experimental; clear color stays host-side)
- Fixture `fixtures/w1/webgpu_begin_render_pass`; native `wasi_webgpu_begin_render_pass`; twin instrument `WasiWebGpuBeginRenderPassInstrumentedTest`
- Guest stub view `23` (not surface); instrument substitutes Cpu offscreen TextureView. Not `[method]gpu-command-encoder.begin-render-pass`; not render-pass-end; not present; not compliance
