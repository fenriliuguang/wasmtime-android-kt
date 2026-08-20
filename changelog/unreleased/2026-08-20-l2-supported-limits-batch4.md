### Code — L2 gpu-supported-limits batch4 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-dynamic-storage-buffers-per-pipeline-layout` / `max-dynamic-uniform-buffers-per-pipeline-layout` / `max-immediate-size` / `max-inter-stage-shader-variables` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxDynamicStorageBuffersPerPipelineLayout` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxDynamicStorageBuffersPerPipelineLayout` (and siblings)
