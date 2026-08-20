### Code — L2 gpu-render-bundle-encoder bind-group and indirect draws guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-bundle-encoder.set-bind-group` / `draw-indirect` / `draw-indexed-indirect` from lift-only stubs to described JNI (encoder + bind-group/buffer reps + index/offset → Dawn/Cpu; 0 reps → stub in the attach)
- Guest constructors still use rep 0; the wraps stub-create an RGBA8 bundle encoder when needed
- New host APIs `renderBundleEncoderSetBindGroup` / `DrawIndirect` / `DrawIndexedIndirect`
