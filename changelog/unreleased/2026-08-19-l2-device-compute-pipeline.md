### Code — L2 gpu-device create-compute-pipeline guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-compute-pipeline` and `create-compute-pipeline-async` from a host-fixed stub to described JNI (`HostArg` shader handle + entry-point string + layout handle + optional label)
- Guest passes entry-point=`main` and label=`l2` with layout=auto; native wrap uses device/shader `rep` when non-zero (shader/layout 0 still stub on the host) and forwards fields into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_create_compute_pipeline` / `_async`; native modules `create_compute_pipeline` / `create_compute_pipeline_async`
