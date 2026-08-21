### Code — L2 begin-render-pass depth + clear (2026-08-21)

- `[method]gpu-command-encoder.begin-render-pass` forwards the first color attachment **plus** `depth-stencil-attachment` (view + depth load/store/clear) **plus** color `clear-value` when present
- Fixture `webgpu_method_begin_render_pass` asserts color clear `(0,0,0,1)` and a depth attachment (clear=1, load=clear, store=store); extra color attachments stay dropped
- JNI `beginRenderPassDescribed` is now `(IIIIIFFFFIIIIF)I`; depth view `0` still means no depth attachment
