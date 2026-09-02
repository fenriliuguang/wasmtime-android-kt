### API — ND-DEFAULT product NativeGpu (2026-09-01)

- `GpuBackends.dawn()` / ServiceLoader `id == "dawn"` / `:android-webgpu` default is in-process NativeGpu (`Store.setWebGpuBackend` still the SPI). androidx JNI leftover is explicit `GpuBackends.dawnJni()` (`id = "dawn-jni"`). Unwired `Store.create` still leaves `request-adapter` **`none`**.
- Default APK **excludes** `libwebgpu_c_bundled.so` and packs recipe `libwebgpu_dawn.so` when present (`native/third_party/dawn-c/out/<abi>/`). One Dawn binary. `dlopen` is best-effort (Cloud / missing NDK → table-backed, Dawn C slots stay 0).
- Product coordinate stays **`0.1.0`** (not pressed). Kotlin SPI unchanged. Leftover table: [`docs/mapping/gap-webgpu-native-dawn.md`](../../docs/mapping/gap-webgpu-native-dawn.md).
