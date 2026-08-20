### Code — L2 gpu-supported-limits batch1 guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-limits.max-bind-groups` / `max-bind-groups-plus-vertex-buffers` / `max-bindings-per-bind-group` / `max-buffer-size` from lift-only stubs to described JNI
- `GpuSupportedLimits { adapter, device }` stores owning adapter or device rep from `gpu-adapter.limits` / `gpu-device.limits`; getters forward both handles to host
- New host APIs on `attachAdapterInfo` for the four limit scalars (Cpu stub 1 / 1L; Dawn reads adapter/device limits)
