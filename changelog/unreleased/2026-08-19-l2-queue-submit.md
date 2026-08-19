### Code — L2 gpu-queue submit guest fields to host (2026-08-19)

- Deepen `[method]gpu-queue.submit` from a host-fixed stub to described JNI (`HostArg` int array: command-buffer handles)
- Guest passes a one-element `list<borrow<gpu-command-buffer>>`; native wrap uses queue `rep` when non-zero and forwards the list into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_queue_submit`; native module `queue_submit`
