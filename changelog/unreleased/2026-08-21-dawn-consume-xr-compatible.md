### Code — L2 dawn consume xr-compatible (2026-08-21)

- Dawn `GPURequestAdapterOptions.requestAdapterWebXROptions` now takes snapshotted `xrCompatible` (`GPURequestAdapterWebXROptions`; absent → none)
- Do not re-cut F6 JNI; fixture `webgpu_method_request_adapter` already lifts `xr-compatible=true`; keep Vulkan `backendType`
- androidx `1.0.0-alpha05` exposes the WebXR options slot (no bare `xrCompatible` on `GPURequestAdapterOptions`)
