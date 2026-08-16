### Code — wasi:webgpu W3 [method]gpu-queue.write-texture (2026-08-16)

- Register `[method]gpu-queue.write-texture` on existing `gpu-queue` / `get-queue` (sync void; L2 adapter → device → queue then host-fixed 1×1 COPY_DST texture write; Guest stub texture u32 ignored)
- Fixture `fixtures/w1/webgpu_method_write_texture`; native `wasi_webgpu_method_write_texture`; twin instrument `WasiWebGpuMethodWriteTextureInstrumentedTest`
- Transitional: no Guest `gpu-texel-copy-texture-info` / `list<u8>`; still void. Not compliance
