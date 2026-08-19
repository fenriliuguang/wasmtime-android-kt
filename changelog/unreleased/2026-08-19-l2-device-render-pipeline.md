### Code — L2 gpu-device create-render-pipeline guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-render-pipeline` and `create-render-pipeline-async` from a host-fixed stub to described JNI (`HostArg` vertex/fragment shader handles + entry-points + format + layout handle + optional label)
- Guest passes vertex entry-point=`vs_main` and label=`l2` with layout=auto; native wrap uses device/shader `rep` when non-zero (shader/layout 0 still stub on the host) and forwards fields into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixtures `webgpu_method_create_render_pipeline` / `_async`; native modules `create_render_pipeline` / `create_render_pipeline_async`
