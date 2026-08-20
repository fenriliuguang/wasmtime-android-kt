### Code — L2 gpu-buffer mapped-range copies guest fields to host (2026-08-20)

- Deepen `[method]gpu-buffer.get-mapped-range-get-with-copy` / `get-mapped-range-set-with-copy` from lift-only stubs to described JNI (buffer handle + offset/size/data → Dawn/Cpu mapped-range read/write)
- Guest `get-buffer` still uses rep 0; native wrap stub-creates a 4-byte buffer; the attach maps before access; export `run` returns harness `1`
- New JNI `call_bytes` (byte[] return); new host API `bufferSetMappedRange`
