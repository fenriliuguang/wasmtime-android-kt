### Code — L2 gpu-supported-limits batch5 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-sampled-textures-per-shader-stage` / `max-samplers-per-shader-stage` / `max-storage-buffer-binding-size` / `max-storage-buffers-in-fragment-stage` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxSampledTexturesPerShaderStage` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxSampledTexturesPerShaderStage` (and siblings)
