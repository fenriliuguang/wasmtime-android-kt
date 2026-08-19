### Code — L2 gpu-command-encoder copy/clear-buffer guest fields to host (2026-08-19)

- Deepen `[method]gpu-command-encoder.copy-buffer-to-buffer` and `clear-buffer` from host-fixed 4-byte copy to described JNI (buffer reps + option offsets/size)
- Guest passes offsets `some(0)` and size `some(4)`; native wrap uses encoder `rep` when non-zero and forwards Dawn ints/longs into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_copy_buffer_to_buffer` / `webgpu_method_clear_buffer`; native modules `copy_buffer_to_buffer` / `clear_buffer`
