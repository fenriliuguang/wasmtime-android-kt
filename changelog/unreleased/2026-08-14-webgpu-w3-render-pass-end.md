### Code — wasi:webgpu W3 render-pass-end dual-register (2026-08-14)

- Dual-register transitional flat `wasi:webgpu/webgpu@0.3.0-rc.2#render-pass-end` (sync void; same L2 as experimental)
- Fixture `fixtures/w1/webgpu_render_pass_end`; native `wasi_webgpu_render_pass_end`; twin instrument `WasiWebGpuRenderPassEndInstrumentedTest`
- Guest stub view `23` then end; instrument substitutes Cpu offscreen TextureView. Not `[method]gpu-render-pass-encoder.end`; not finish/submit/present; not compliance
