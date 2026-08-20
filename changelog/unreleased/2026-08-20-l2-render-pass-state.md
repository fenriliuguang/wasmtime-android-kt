### Code — L2 gpu-render-pass state setters guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-pass-encoder.set-viewport` / `set-scissor-rect` / `set-blend-constant` / `set-stencil-reference` from lift-only stubs to described JNI (pass handle + floats / ints / color / reference → Dawn/Cpu)
- Guest `get-pass` still uses rep 0; the wraps stub-build a clear render pass when needed
- `HostArg` gains `Float` / `Double` scalars; new host APIs `renderPassSetViewport` / `SetScissorRect` / `SetBlendConstant` / `SetStencilReference`
