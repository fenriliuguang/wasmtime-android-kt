### Code — L2 gpu-queue write-buffer guest fields to host (2026-08-19)

- Deepen `[method]gpu-queue.write-buffer-with-copy` from a host-fixed stub to described JNI (`HostArg` bytes: guest `list<u8>` plus buffer offset)
- Guest passes 4-byte data `l2\\0\\0` at offset 0; native wrap uses queue/buffer `rep` when non-zero and forwards the payload into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_write_buffer`; native module `write_buffer`
