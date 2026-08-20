### Code — L2 gpu-supported-features.has + wgsl-language-features.has guest fields to host (2026-08-20)

- Deepen `[method]gpu-supported-features.has` / `[method]wgsl-language-features.has` from lift-only stubs to described JNI with guest adapter handle (features) and feature name string
- `GpuSupportedFeatures` stores the owning adapter rep from `gpu-adapter.features` / `gpu-device.features`; `WgslLanguageFeatures` keeps `{ gpu: 0 }` while `has` forwards the guest feature name
- New host APIs `supportedFeaturesHas` / `wgslLanguageFeaturesHas` on `attachAdapterInfo` / `attachRequestAdapter`
