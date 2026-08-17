### Code — S6 [method]gpu-device.create-command-encoder returns own<gpu-command-encoder> (2026-08-17)

- Replace transitional u32 with WIT-shaped `(borrow<gpu-device>, option<gpu-command-encoder-descriptor>) -> own<gpu-command-encoder>`
- Guest passes descriptor=none; host table stores L2 handle in `GpuCommandEncoder.rep`; drops the own handle; export `run` returns harness `1`
- Fixture `fixtures/w1/webgpu_method_create_command_encoder`; native module `wasi_webgpu_method/create_command_encoder`; twin instrument `WasiWebGpuMethodCreateCommandEncoderInstrumentedTest`
