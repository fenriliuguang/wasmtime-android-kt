### Code — wasi:webgpu W3 [method]gpu-command-encoder.finish (2026-08-15)

- Register `[method]gpu-command-encoder.finish` on existing `gpu-command-encoder` / `get-encoder` (sync; L2 adapter → device → encoder → finish, same u32 as flat `command-encoder-finish`)
- Fixture `fixtures/w1/webgpu_method_command_encoder_finish`; native `wasi_webgpu_method_command_encoder_finish`; twin instrument `WasiWebGpuMethodCommandEncoderFinishInstrumentedTest`
- Transitional: no descriptor option. Flat name kept. Not compliance
