### Code — L2 gpu-uncaptured-error-event.error guest fields to host (2026-08-20)

- Deepen `[method]gpu-uncaptured-error-event.error` from omit-lane lift-only stub to described JNI with guest device handle
- `GpuUncapturedErrorEvent` stores device rep; returned `GpuError` carries the validated device handle for kind/message getters
- New host API `uncapturedErrorEventError` wired through `attachDeviceInfoError`
