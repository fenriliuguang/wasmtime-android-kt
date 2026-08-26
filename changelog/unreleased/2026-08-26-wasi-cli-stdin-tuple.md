### Code — WASI 0.3 wasi:cli stdin official tuple (2026-08-26)

- Promote `wasi:cli/stdin@0.3.0#read-via-stream` from transitional `func() -> stream<u8>` to official `tuple<stream<u8>, future<result<_, error-code>>>` (ok path)
- Official names: `read-via-stream`, `error-code` (`unknown` only). Guest still returns nbytes (3 for `IN\n`) after future ok
- Native `wasi_cli_stdin`; device `WasiCliStdinInstrumentedTest`
