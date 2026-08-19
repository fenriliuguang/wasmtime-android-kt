### Code — L2 gpu-queue write-texture guest fields to host (2026-08-19)

- Deepen `[method]gpu-queue.write-texture-with-copy` from a host-fixed stub to described JNI (`HostArg` bytes: guest `list<u8>` plus copy width/height/bytes-per-row)
- Guest passes 4-byte data `l2\\0\\0`, `bytes-per-row=4`, size 1×1×1; native wrap uses queue/texture `rep` when non-zero and forwards the payload into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_write_texture`; native module `write_texture`
