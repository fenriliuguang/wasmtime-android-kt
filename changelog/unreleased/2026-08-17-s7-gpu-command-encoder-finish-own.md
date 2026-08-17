### Code — S7 [method]gpu-command-encoder.finish returns own<gpu-command-buffer> (2026-08-17)

- Replace transitional u32 with WIT-shaped `(borrow<gpu-command-encoder>, option<gpu-command-buffer-descriptor>) -> own<gpu-command-buffer>`
- Guest passes descriptor=none; host table stores L2 handle in `GpuCommandBuffer.rep`; drops the own handle; export `run` returns harness `1`
- Fixture `fixtures/w1/webgpu_method_command_encoder_finish`; native module `wasi_webgpu_method/command_encoder_finish`; twin instrument `WasiWebGpuMethodCommandEncoderFinishInstrumentedTest`
