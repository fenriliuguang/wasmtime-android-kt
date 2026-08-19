### Code — L2 gpu-compute-pass-encoder dispatch-indirect guest fields to host (2026-08-19)

- Deepen `[method]gpu-compute-pass-encoder.dispatch-workgroups-indirect` from host-fixed rebuild+dispatch(1,1,1) to described JNI (pass/buffer reps + offset)
- Guest borrows the indirect buffer with offset `0`; native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_compute_pass_dispatch_workgroups_indirect`; native module `compute_pass_dispatch_workgroups_indirect`
