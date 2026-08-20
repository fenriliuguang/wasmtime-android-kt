### Code — L2 gpu-supported-limits batch9 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-vertex-buffer-array-stride` / `max-vertex-buffers` / `min-storage-buffer-offset-alignment` / `min-uniform-buffer-offset-alignment` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxVertexBufferArrayStride` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxVertexBufferArrayStride` (and siblings)
