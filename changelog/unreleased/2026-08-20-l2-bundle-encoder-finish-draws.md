### Code — L2 gpu-render-bundle-encoder finish and draws guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-bundle-encoder.finish` / `draw` / `draw-indexed` from lift-only stubs to described JNI (encoder handle + label / draw counts → Dawn/Cpu; finish stores the bundle rep on the guest resource)
- Guest `get-render-bundle-encoder` still uses rep 0; the wraps stub-create an RGBA8 bundle encoder when needed
- New host APIs `renderBundleEncoderFinish` / `Draw` / `DrawIndexed`
