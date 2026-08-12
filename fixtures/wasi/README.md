# WASI 0.3 package smokes

## `wasi:random` — `get-random-u64`

Guest export: `run: func() -> u64`  
Host: `wasi:random/random@0.3.0#get-random-u64`（CSPRNG；钉 Wasmtime P3 WIT `0.3.0`）

成功：链接 + 调用不 trap（返回任意 `u64`）。

```powershell
wasm-tools parse fixtures/wasi/random_u64.wat -o fixtures/wasi/random_u64.wasm
```

## `wasi:clocks` — `monotonic-clock.now`

Guest export: `run: func() -> u64`  
Host: `wasi:clocks/monotonic-clock@0.3.0#now`（进程内 `Instant` 纪元 → 非递减 mark；钉 `@0.3.0`）

成功：两次调用 `second >= first`（单调）。本切片不含 `wait-until` / `wait-for` / `system-clock` / timezone。

```powershell
wasm-tools parse fixtures/wasi/monotonic_now.wat -o fixtures/wasi/monotonic_now.wasm
```

## `wasi:cli` — `stdout.write-via-stream`

Guest export: `run: func() -> u32`（字节数）  
Host: `wasi:cli/stdout@0.3.0#write-via-stream`（`CollectConsumer` 管道；钉 `@0.3.0`）

**过渡签名：** `func(data: stream<u8>) -> future<u32>`（与根 import `take` 同形）。官方 WIT 为 `future<result<_, error-code>>`；手写 WAT 枚举结果另切片。

成功：guest 写入 `OUT\n`（4 字节）后 `run` 返回 `4`。本切片不含 stdin / stderr / `wasi:cli/command`。

```powershell
wasm-tools parse fixtures/wasi/cli_stdout.wat -o fixtures/wasi/cli_stdout.wasm
```
