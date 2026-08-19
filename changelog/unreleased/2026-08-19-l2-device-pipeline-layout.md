### Code — L2 gpu-device create-pipeline-layout guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-pipeline-layout` from a host-fixed stub to described JNI (`HostArg` int array of bind-group-layout handles + optional label)
- Guest passes empty `bind-group-layouts` and label=`l2`; native wrap uses device `rep` when non-zero and forwards the list/label into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_create_pipeline_layout`; native module `create_pipeline_layout`
