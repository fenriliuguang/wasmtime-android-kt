### Code — wasi:webgpu W3 [method]gpu-command-encoder.copy-buffer-to-buffer (2026-08-16)

- Register `[method]gpu-command-encoder.copy-buffer-to-buffer` on existing `gpu-command-encoder` / `get-encoder` (sync void; L2 adapter → device → encoder then host-fixed 4-byte copy; Guest stub source/destination ignored)
- Fixture `fixtures/w1/webgpu_method_copy_buffer_to_buffer`; native `wasi_webgpu_method_copy_buffer_to_buffer`; twin instrument `WasiWebGpuMethodCopyBufferToBufferInstrumentedTest`
- Transitional: Guest returns stub 31. Not compliance
