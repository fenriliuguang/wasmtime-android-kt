### Code — L2 gpu-device create-sampler guest fields to host (2026-08-19)

- Deepen `[method]gpu-device.create-sampler` from host-fixed default sampler to described JNI (`deviceCreateSamplerDescribed`)
- Guest passes `some(gpu-sampler-descriptor)` with address-mode-u=repeat and mag/min-filter=linear; native wrap forwards Dawn ints into `WasiWebGpuHost`; drops own; export `run` returns harness `1`
- Fixture `webgpu_method_create_sampler`; native module `create_sampler`; twin instrument `WasiWebGpuMethodCreateSamplerInstrumentedTest`
