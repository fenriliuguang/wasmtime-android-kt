### Code — S6+ unmap / write-*-with-copy cluster (2026-08-18)

- Cut three remaining buffer/queue methods that still used transitional `u32` or void: `unmap`, `write-buffer` (now WIT `write-buffer-with-copy`), `write-texture` (now WIT `write-texture-with-copy`)
- Guest passes WIT result / borrow + `list<u8>` / texel copy info; drops extra owns; export `run` returns harness `1`; L2 stays host-fixed map-then-unmap / 4-byte write / 1×1 write
- Fixtures `webgpu_method_buffer_unmap` / `webgpu_method_write_buffer` / `webgpu_method_write_texture`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
