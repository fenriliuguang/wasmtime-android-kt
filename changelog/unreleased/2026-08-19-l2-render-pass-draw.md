### Code — L2 gpu-render-pass-encoder draw family guest fields to host (2026-08-19)

- Deepen `[method]gpu-render-pass-encoder.draw`, `draw-indexed`, and rider `end` from host-fixed rebuild+draw(3) to described JNI (pass rep + counts)
- Guest passes vertex-count/index-count `3` and other options none; native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_render_pass_draw` / `webgpu_method_render_pass_draw_indexed` / `webgpu_method_render_pass_end`; native modules `render_pass_draw` / `render_pass_draw_indexed` / `render_pass_end`
