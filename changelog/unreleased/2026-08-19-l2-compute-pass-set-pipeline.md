### Code — L2 gpu-compute-pass-encoder set-pipeline guest fields to host (2026-08-19)

- Deepen `[method]gpu-compute-pass-encoder.set-pipeline` from host-fixed rebuild+stub pipeline to described JNI (pass + pipeline reps)
- Guest borrows the pipeline; native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_compute_pass_set_pipeline`; native module `compute_pass_set_pipeline`
