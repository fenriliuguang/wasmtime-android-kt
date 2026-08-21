### Code — L2 create-texture mip / sample / dimension (2026-08-21)

- `[method]gpu-device.create-texture` forwards guest `mip-level-count` / `sample-count` / `dimension` with existing size/format/usage; `view-formats` and label stay dropped
- Fixture `webgpu_method_create_texture` asserts mip=2, sample=1, dimension=d2 (Dawn `TextureDimension` 2D=2)
- JNI `deviceCreateTextureDescribed` is now `(IIIIIIIII)I`; absent mip/sample default to 1, absent dimension to 2D
