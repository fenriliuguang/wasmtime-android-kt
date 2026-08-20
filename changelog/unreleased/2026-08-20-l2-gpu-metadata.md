### Code — L2 gpu metadata getters guest fields to host (2026-08-20)

- Deepen `[method]gpu.get-preferred-canvas-format` / `[method]gpu.wgsl-language-features` from cm-fixed stubs to described JNI through `attachRequestAdapter`
- `get-preferred-canvas-format` returns host texture-format ordinal via `gpuGetPreferredCanvasFormat` (Cpu/Dawn rgba8unorm)
- `gpu.wgsl-language-features` calls `gpuWgslLanguageFeatures` validate before pushing local `WgslLanguageFeatures { gpu: 0 }`
