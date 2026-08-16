### Code — wasi:webgpu W3 [method]gpu-compute-pass-encoder.end (2026-08-16)

- Register WIT `gpu-compute-pass-encoder` + `get-compute-pass` + `[method]gpu-compute-pass-encoder.end` (sync void; L2 adapter → device → encoder → begin-compute-pass then end; Guest stub pass ignored; do not reuse `get-pass`)
- Fixture `fixtures/w1/webgpu_method_compute_pass_end`; native `wasi_webgpu_method_compute_pass_end`; twin instrument `WasiWebGpuMethodComputePassEndInstrumentedTest`
- Transitional: Guest returns stub 79. Not compliance
