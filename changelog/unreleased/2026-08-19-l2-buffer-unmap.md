### Code — L2 gpu-buffer unmap guest fields to host (2026-08-19)

- Deepen `[method]gpu-buffer.unmap` from host-fixed rebuild+map-then-unmap to described JNI (buffer rep)
- Guest unmaps; native wrap uses buffer `rep` when non-zero and forwards the handle into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_buffer_unmap`; native module `buffer_unmap`
