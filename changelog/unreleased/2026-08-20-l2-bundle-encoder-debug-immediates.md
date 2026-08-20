### Code — L2 gpu-render-bundle-encoder debug and set-immediates guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-bundle-encoder.push-debug-group` / `pop-debug-group` / `insert-debug-marker` / `set-immediates` from lift-only stubs to described JNI (encoder handle + labels / immediates bytes → Dawn/Cpu)
- Guest `get-render-bundle-encoder` still uses rep 0; the wraps stub-create an RGBA8 bundle encoder when needed
- New host APIs `renderBundleEncoderPushDebugGroup` / `PopDebugGroup` / `InsertDebugMarker` / `SetImmediates`; this empties the default lift-only lane (68 → 0)
