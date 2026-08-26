### Code — WASI 0.3 wasi:filesystem open-at .. access (2026-08-26)

- Guest `open-at("..")` yields official `error-code.access` (u8 disc + payload). Happy path still `open-at("p3fs.txt")` + 4-byte `P3FS`. Completes **G-fs-open**. No `wasmtime-wasi`
- Host join already rejected `..` / absolute / NUL; this cut makes it guest-visible. Native `open_at_dotdot_returns_access`
- Device `WasiFilesystemPreopenInstrumentedTest` still 4-byte `P3FS` (access check is in the same guest `run`)
