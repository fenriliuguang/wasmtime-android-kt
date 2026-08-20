### Code — L2 gpu-render-pass debug and set-immediates guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-pass-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` / `set-immediates` from lift-only stubs to described JNI (pass handle + labels / immediates bytes → Dawn/Cpu)
- Guest `get-pass` still uses rep 0; the wraps stub-build a clear render pass when needed
- New host APIs `renderPassPushDebugGroup` / `PopDebugGroup` / `InsertDebugMarker` / `SetImmediates`; this completes the `gpu-render-pass-encoder` default lift-only lane
