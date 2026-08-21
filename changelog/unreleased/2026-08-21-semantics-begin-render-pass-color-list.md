### Code — leftover descriptor semantics begin-render-pass color list (2026-08-21)

- JNI `beginRenderPassDescribed` now packs **all** color attachments (view / load / store / optional clear bits; view 0 = none) into existing `RenderPassDescriptor.colorAttachments`
- Fixture `webgpu_method_begin_render_pass` passes two color views (clears `0,0,0,1` and `1,0,0,1`); native smoke asserts both lifted attachments
- Dawn already maps the Kotlin color list into `GPURenderPassDescriptor`; empty extra list stays valid
