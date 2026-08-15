### Code — wasi:webgpu W3 [method]gpu-queue.write-buffer (2026-08-15)

- Register `[method]gpu-queue.write-buffer` on existing `gpu-queue` / `get-queue` (sync void; Guest stub buffer u32; JNI creates a real buffer then host-fixed 4-byte write)
- Fixture `fixtures/w1/webgpu_method_write_buffer`; native `wasi_webgpu_method_write_buffer`; twin instrument `WasiWebGpuMethodWriteBufferInstrumentedTest`
- Transitional: no Guest `list<u8>` / offset; not proposal `gpu-buffer` borrow. Not compliance
