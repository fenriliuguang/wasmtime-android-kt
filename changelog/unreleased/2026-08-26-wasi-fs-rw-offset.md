### Code — WASI 0.3 wasi:filesystem read/write filesize offset (2026-08-26)

- Promote `[method]descriptor.write-via-stream` / `read-via-stream` to official `offset: filesize` (`u64`). Smoke still writes/reads `P3FS` at offset `0`. No `wasmtime-wasi`
- Offset `0` write still replaces the file; non-zero splices. Completes **G-fs-shape**. Directory `open-at` remains P1-FS3
- Native `wasi_filesystem_preopen`; device `WasiFilesystemPreopenInstrumentedTest` still 4-byte `P3FS`
