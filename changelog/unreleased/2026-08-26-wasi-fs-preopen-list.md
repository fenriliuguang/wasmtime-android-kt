### Code — WASI 0.3 wasi:filesystem get-directories list tuple (2026-08-26)

- Promote `wasi:filesystem/preopens@0.3.0#get-directories` from `own<descriptor>` to official `list<tuple<own<descriptor>, string>>` (length 1, name `p3fs.txt`). Guest uses index 0. No `wasmtime-wasi`
- r/w-via-stream polarity unchanged (cli shapes, no `filesize` offset — P1-FS2). Still file preopen, not directory `open-at` (P1-FS3)
- Native `wasi_filesystem_preopen`; device `WasiFilesystemPreopenInstrumentedTest` still 4-byte `P3FS`
