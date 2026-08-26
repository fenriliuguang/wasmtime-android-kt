### Code — WASI 0.3 wasi:filesystem Android sandbox preopen (2026-08-26)

- Official names: `wasi:filesystem/preopens@0.3.0#get-directories`, `wasi:filesystem/types@0.3.0` `resource descriptor`, `[method]descriptor.write-via-stream` / `read-via-stream`, `error-code` (`unknown` + `access`). No `wasmtime-wasi`
- Subset (not a full world): `get-directories` returns `own<descriptor>` (not `list<tuple<descriptor, string>>`); write takes `stream<u8>` like cli stdout; read returns `tuple<stream, future<result>>` like stdin. No `open-at` / directory list. Sandbox: `temp_dir()/wasmtime-android-kt-wasi-fs` (Android: app-private cache via `TMPDIR`); reject `..` / absolute / NUL; not `/sdcard`
- Native `wasi_filesystem_preopen`; device `WasiFilesystemPreopenInstrumentedTest`
