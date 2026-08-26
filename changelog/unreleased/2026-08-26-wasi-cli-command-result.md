### Code — WASI 0.3 wasi:cli/run official empty result (2026-08-26)

- Export `wasi:cli/run@0.3.0#run` as `async func() -> result` (empty ok). Root `run -> u32` harness stays 0=ok for `callRunConcurrent`
- Still uses existing stdout `write-via-stream`; not a full command world (no fs/sockets/exit)
- Native `wasi_cli_command` covers harness + official export; device `WasiCliCommandInstrumentedTest`
