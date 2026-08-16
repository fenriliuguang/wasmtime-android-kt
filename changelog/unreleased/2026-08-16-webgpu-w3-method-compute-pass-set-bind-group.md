### Code — wasi:webgpu W3 [method]gpu-compute-pass-encoder.set-bind-group (2026-08-16)

- Register `[method]gpu-compute-pass-encoder.set-bind-group` on existing `gpu-compute-pass-encoder` / `get-compute-pass` (sync void; L2 adapter → device → encoder → begin-compute-pass then host-fixed empty bind-group at index 0; Guest stub bind-group ignored)
- Fixture `fixtures/w1/webgpu_method_compute_pass_set_bind_group`; native `wasi_webgpu_method_compute_pass_set_bind_group`; twin instrument `WasiWebGpuMethodComputePassSetBindGroupInstrumentedTest`
- Transitional: Guest returns stub 67. Not compliance
