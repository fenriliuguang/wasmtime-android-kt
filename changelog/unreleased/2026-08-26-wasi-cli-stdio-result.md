### Code — WASI 0.3 wasi:cli stdout/stderr official result (2026-08-26)

- Promote `wasi:cli/stdout@0.3.0` and `stderr@0.3.0` `write-via-stream` from transitional `future<u32>` to official `future<result<_, error-code>>` (ok path)
- Official names: `write-via-stream`, `error-code` (this cut: `unknown` only). Guest `run` still returns 4 after ok so existing instruments keep asserting byte count
- `cli_command` import updated to the same stdout result type (command `run -> u32` / empty result remains W5)
