### Code — L2 gpu-render-pass-encoder pipeline and buffers guest fields to host (2026-08-19)

- Deepen `[method]gpu-render-pass-encoder.set-pipeline`, `set-vertex-buffer`, and `set-index-buffer` from host-fixed rebuild+stub to described JNI (pass/pipeline/buffer reps + slot/format + option offset/size)
- Guest borrows pipeline or buffer (slot 0 / uint16, offset/size none); native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_render_pass_set_pipeline` / `webgpu_method_render_pass_set_vertex_buffer` / `webgpu_method_render_pass_set_index_buffer`; native modules `render_pass_set_pipeline` / `render_pass_set_vertex_buffer` / `render_pass_set_index_buffer`
