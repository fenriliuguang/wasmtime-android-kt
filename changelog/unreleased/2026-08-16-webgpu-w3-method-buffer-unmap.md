### Code — wasi:webgpu W3 [method]gpu-buffer.unmap (2026-08-16)

- Register `[method]gpu-buffer.unmap` on existing `gpu-buffer` / `get-buffer` (sync void; L2 adapter → device → host-fixed MAP_READ buffer then map + unmap; Guest stub buffer ignored)
- Fixture `fixtures/w1/webgpu_method_buffer_unmap`; native `wasi_webgpu_method_buffer_unmap`; twin instrument `WasiWebGpuMethodBufferUnmapInstrumentedTest`
- Transitional: Guest returns stub 31. Not compliance
