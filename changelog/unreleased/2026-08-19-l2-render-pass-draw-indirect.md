### Code — L2 gpu-render-pass-encoder indirect-draw family guest fields to host (2026-08-19)

- Deepen `[method]gpu-render-pass-encoder.draw-indirect` and `draw-indexed-indirect` from host-fixed rebuild+draw(3) to described JNI (pass/buffer reps + offset)
- Guest borrows the indirect buffer with offset `0`; native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_render_pass_draw_indirect` / `webgpu_method_render_pass_draw_indexed_indirect`; native modules `render_pass_draw_indirect` / `render_pass_draw_indexed_indirect`
