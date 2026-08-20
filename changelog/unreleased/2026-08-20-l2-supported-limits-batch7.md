### Code — L2 gpu-supported-limits batch7 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-storage-textures-per-shader-stage` / `max-texture-array-layers` / `max-texture-dimension1-d` / `max-texture-dimension2-d` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxStorageTexturesPerShaderStage` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxStorageTexturesPerShaderStage` (and siblings)
