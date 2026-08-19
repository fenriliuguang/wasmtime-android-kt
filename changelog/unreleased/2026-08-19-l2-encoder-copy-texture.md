### Code — L2 gpu-command-encoder texture-copy guest fields to host (2026-08-19)

- Deepen `[method]gpu-command-encoder.copy-buffer-to-texture`, `copy-texture-to-buffer`, and `copy-texture-to-texture` from host-fixed 4-byte buffer copy to described JNI (encoder/buffer/texture reps + extent)
- Guest already passes size 1×1×1; native wrap uses encoder `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_copy_buffer_to_texture` / `webgpu_method_copy_texture_to_buffer` / `webgpu_method_copy_texture_to_texture`; native modules `copy_buffer_to_texture` / `copy_texture_to_buffer` / `copy_texture_to_texture`
