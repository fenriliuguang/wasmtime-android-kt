### Code — wasi:webgpu W3 [method]gpu-command-encoder.begin-compute-pass (2026-08-16)

- Register `[method]gpu-command-encoder.begin-compute-pass` on existing `gpu-command-encoder` / `get-encoder` (sync; L2 adapter → device → encoder then begin-compute-pass with host-default descriptor; Guest does not pass descriptor)
- Fixture `fixtures/w1/webgpu_method_begin_compute_pass`; native `wasi_webgpu_method_begin_compute_pass`; twin instrument `WasiWebGpuMethodBeginComputePassInstrumentedTest`
- Transitional: still u32, not `gpu-compute-pass-encoder` resource. Not compliance
