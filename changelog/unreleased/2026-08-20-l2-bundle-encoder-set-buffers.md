### Code — L2 gpu-render-bundle-encoder pipeline and buffers guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-bundle-encoder.set-pipeline` / `set-vertex-buffer` / `set-index-buffer` from lift-only stubs to described JNI (encoder + pipeline/buffer reps + slot/format/offset/size → Dawn/Cpu; 0 reps → stub in the attach)
- Guest constructors still use rep 0; the wraps stub-create an RGBA8 bundle encoder when needed
- New host APIs `renderBundleEncoderSetPipeline` / `SetVertexBuffer` / `SetIndexBuffer`
