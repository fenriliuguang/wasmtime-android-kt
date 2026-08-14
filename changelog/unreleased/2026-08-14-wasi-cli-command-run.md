### Code — WASI 0.3 wasi:cli/command async run smoke (2026-08-14)

- Command-shaped guest exports root `run: async func() -> u32` (0 = ok; official empty `result` deferred)
- Imports existing `wasi:cli/stdout@0.3.0#write-via-stream` subset (or random fallback); fixture `fixtures/wasi/cli_command`; native `wasi_cli_command`; twin `WasiCliCommandInstrumentedTest`
- Not full command world (no fs/sockets/exit); not official result/error-code
