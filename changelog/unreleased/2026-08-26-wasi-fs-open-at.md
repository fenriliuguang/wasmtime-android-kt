### Code — WASI 0.3 wasi:filesystem directory preopen + open-at (2026-08-26)

- Preopen is the sandbox **directory** (`get-directories` name `"."`). Add `[method]descriptor.open-at(path) -> result<descriptor, error-code>`; smoke opens `"p3fs.txt"` and r/w the child. No `wasmtime-wasi`
- Path join stays `filesystem_sandbox_join` (reject `..` / absolute / NUL as `access` on the host; guest-visible `..` is P1-FS4). Flags (`path-flags` / `open-flags`) stay **G-fs-full**
- Native `wasi_filesystem_preopen`; device `WasiFilesystemPreopenInstrumentedTest` still 4-byte `P3FS`
