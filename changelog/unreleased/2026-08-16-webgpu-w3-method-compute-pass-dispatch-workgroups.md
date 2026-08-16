### Code — wasi:webgpu W3 [method]gpu-compute-pass-encoder.dispatch-workgroups (2026-08-16)

- Register `[method]gpu-compute-pass-encoder.dispatch-workgroups` on existing `gpu-compute-pass-encoder` / `get-compute-pass` (sync void; L2 adapter → device → encoder → begin-compute-pass then host-fixed set-pipeline + empty bind-group 0 + dispatch 1×1×1; Guest workgroup counts ignored)
- Fixture `fixtures/w1/webgpu_method_compute_pass_dispatch_workgroups`; native `wasi_webgpu_method_compute_pass_dispatch_workgroups`; twin instrument `WasiWebGpuMethodComputePassDispatchWorkgroupsInstrumentedTest`
- Transitional: Guest returns stub 79. Not compliance
