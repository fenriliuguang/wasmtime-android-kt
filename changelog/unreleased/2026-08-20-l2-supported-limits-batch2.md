### Code — L2 gpu-supported-limits batch2 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-color-attachment-bytes-per-sample` / `max-color-attachments` / `max-compute-invocations-per-workgroup` / `max-compute-workgroup-size-x` from lift-only stubs to described JNI
- Reuse `GpuSupportedLimits { adapter, device }` handles through `l2_supported_limits_handles` into host `supportedLimitsMaxColorAttachmentBytesPerSample` family
- Cpu stubs stay 1 / 1L; Dawn reads adapter or device `limits.maxColorAttachmentBytesPerSample` (and siblings)
