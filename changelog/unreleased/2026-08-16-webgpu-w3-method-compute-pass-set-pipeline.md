### Code — wasi:webgpu W3 [method]gpu-compute-pass-encoder.set-pipeline (2026-08-16)

- Register `[method]gpu-compute-pass-encoder.set-pipeline` on existing `gpu-compute-pass-encoder` / `get-compute-pass` (sync void; L2 adapter → device → encoder → begin-compute-pass then host-fixed stub shader + empty layout compute pipeline; Guest stub pipeline ignored)
- Fixture `fixtures/w1/webgpu_method_compute_pass_set_pipeline`; native `wasi_webgpu_method_compute_pass_set_pipeline`; twin instrument `WasiWebGpuMethodComputePassSetPipelineInstrumentedTest`
- Transitional: Guest returns stub 73. Not compliance
