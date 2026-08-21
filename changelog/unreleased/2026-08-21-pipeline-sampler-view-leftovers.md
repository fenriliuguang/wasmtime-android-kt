### Code — L2 sampler / view leftover guest fields to host (2026-08-21)

- `[method]gpu-device.create-sampler` forwards leftover `address-mode-v/w`, `mipmap-filter`, lod clamps, and `compare` with existing mag/min/`address-mode-u`; `max-anisotropy` and label stay dropped
- `[method]gpu-texture.create-view` forwards leftover `format`, mip window, and array layers with existing dimension/aspect; usage/swizzle/label stay dropped
- Fixtures `webgpu_method_create_sampler` / `webgpu_method_texture_create_view`; JNI `deviceCreateSamplerDescribed` is now `(IIIIIIIIIFIF)I`, `textureCreateViewDescribed` is `(IIIIIIII)I` (`-1` = absent mip/array count)
