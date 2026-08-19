### Code — L2 gpu-compute-pass-encoder dispatch family guest fields to host (2026-08-19)

- Deepen `[method]gpu-compute-pass-encoder.dispatch-workgroups` and rider `end` from host-fixed rebuild+dispatch(1,1,1) to described JNI (pass rep + workgroup counts)
- Guest passes x=1 and y/z some(1); native wrap uses pass `rep` when non-zero and forwards Dawn ints into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_compute_pass_dispatch_workgroups` / `webgpu_method_compute_pass_end`; native modules `compute_pass_dispatch_workgroups` / `compute_pass_end`
