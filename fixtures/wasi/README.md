# WASI 0.3 package smokes

## `wasi:random` — `get-random-u64`

Guest export: `run: func() -> u64`  
Host: `wasi:random/random@0.3.0#get-random-u64`（CSPRNG；钉 Wasmtime P3 WIT `0.3.0`）

成功：链接 + 调用不 trap（返回任意 `u64`）。

```powershell
wasm-tools parse fixtures/wasi/random_u64.wat -o fixtures/wasi/random_u64.wasm
```

## `wasi:random` — `get-random-bytes`

Guest export: `run: func() -> u64`（把 8 字节 LE 打成 u64）
Host: `wasi:random/random@0.3.0#get-random-bytes`（CSPRNG `list<u8>`；host 长度上限 4096；钉 `@0.3.0`）

成功：两次 `run` 返回值不同（非常量 stub）。

```powershell
wasm-tools parse fixtures/wasi/random_bytes.wat -o fixtures/wasi/random_bytes.wasm
```

## `wasi:clocks` — `monotonic-clock.now`

Guest export: `run: func() -> u64`  
Host: `wasi:clocks/monotonic-clock@0.3.0#now`（进程内 `Instant` 纪元 → 非递减 mark；钉 `@0.3.0`）

成功：两次调用 `second >= first`（单调）。

```powershell
wasm-tools parse fixtures/wasi/monotonic_now.wat -o fixtures/wasi/monotonic_now.wasm
```

## `wasi:cli` — `stdout.write-via-stream`

Guest export: `run: func() -> u32`（ok 后返回写入字节数 4）  
Host: `wasi:cli/stdout@0.3.0#write-via-stream`（`CollectConsumer` 管道；钉 `@0.3.0`）

官方签名：`func(data: stream<u8>) -> future<result<_, error-code>>`。ok 路径 guest 写 `OUT\n` 后 `run` 返回 `4`。**P010-CLIERR：** NUL 字节 → `error-code.illegal-byte-sequence`（`cli_stdout_err`；枚举含 `unknown`/`io`/`illegal-byte-sequence`/`pipe`）。

成功：ok 路径返回 `4`；err 路径 guest 见到 `illegal-byte-sequence` 后 `run` 返回 `1`。

```powershell
wasm-tools parse fixtures/wasi/cli_stdout.wat -o fixtures/wasi/cli_stdout.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/cli_stdout.wasm
wasm-tools parse fixtures/wasi/cli_stdout_err.wat -o fixtures/wasi/cli_stdout_err.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/cli_stdout_err.wasm
```

## `wasi:cli` — `stderr.write-via-stream`

Guest export: `run: func() -> u32`（ok 后返回写入字节数 4）  
Host: `wasi:cli/stderr@0.3.0#write-via-stream`（与 stdout 共用管道；钉 `@0.3.0`）

官方签名：`func(data: stream<u8>) -> future<result<_, error-code>>`（ok 路径；`error-code` 含 `unknown`/`io`/`illegal-byte-sequence`/`pipe`，NUL → `illegal-byte-sequence`）。

成功：guest 写入 `ERR\n`（4 字节）且 future 为 ok 后 `run` 返回 `4`。

```powershell
wasm-tools parse fixtures/wasi/cli_stderr.wat -o fixtures/wasi/cli_stderr.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/cli_stderr.wasm
```

## `wasi:clocks` — `monotonic-clock.wait-for`

Guest export: `run: async func() -> u32`（先 `wait-for` ~2ms，再返回 `1`）  
Host: `wasi:clocks/monotonic-clock@0.3.0#wait-for`（`func_wrap_concurrent` + oneshot / helper-thread sleep；钉 `@0.3.0`）

成功：`run` 经 `run_concurrent` / `call_async` 返回 `1`。本切片不含 `wait-until` / `system-clock` / timezone。

```powershell
wasm-tools parse fixtures/wasi/monotonic_wait_for.wat -o fixtures/wasi/monotonic_wait_for.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/monotonic_wait_for.wasm
```

## `wasi:clocks` — `monotonic-clock.wait-until`

Guest export: `run: async func() -> u32`（`now` → +2ms → `wait-until`，再返回 `1`）  
Host: `wasi:clocks/monotonic-clock@0.3.0#wait-until`（与 `#now` 同 `Instant` 纪元；`func_wrap_concurrent` + oneshot / helper-thread sleep；钉 `@0.3.0`）

成功：`run` 经 `run_concurrent` / `call_async` 返回 `1`。本切片不含 `system-clock` / timezone。

```powershell
wasm-tools parse fixtures/wasi/monotonic_wait_until.wat -o fixtures/wasi/monotonic_wait_until.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/monotonic_wait_until.wasm
```

## `wasi:clocks` — `monotonic-clock.wait-until`

Guest export: `run: async func() -> u32`（`now` → 加 ~2ms → `wait-until` 该 instant，再返回 `1`）  
Host: `wasi:clocks/monotonic-clock@0.3.0#wait-until`（与 `#now` 共享 `Instant` 纪元；`func_wrap_concurrent` + oneshot / helper-thread sleep `max(0, when - now)`，1s 上限；钉 `@0.3.0`）

成功：`run` 经 `run_concurrent` / `call_async` 返回 `1`。`system-clock` 见下节；timezone 另切片。

```powershell
wasm-tools parse fixtures/wasi/monotonic_wait_until.wat -o fixtures/wasi/monotonic_wait_until.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/monotonic_wait_until.wasm
```

## `wasi:cli` — `stdin.read-via-stream`

Guest export: `run: func() -> u32`（字节数）  
Host: `wasi:cli/stdin@0.3.0#read-via-stream`（`StreamReader::new` 产出 `IN\n`；钉 `@0.3.0`）

官方签名：`func() -> tuple<stream<u8>, future<result<_, error-code>>>`（ok 路径；`error-code` 本刀仅 `unknown`）。

成功：guest `stream.read` 且 future 为 ok 后 `run` 返回 `3`。

```powershell
wasm-tools parse fixtures/wasi/cli_stdin.wat -o fixtures/wasi/cli_stdin.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/cli_stdin.wasm
```

## `wasi:clocks` — `system-clock.now`

Guest export: `run: func() -> u64`（unix 秒，取自 `instant.seconds`）  
Host: `wasi:clocks/system-clock@0.3.0#now`（`SystemTime` → official `instant` `{seconds: s64, nanoseconds: u32}`；钉 `@0.3.0`）

成功：返回值落在合理 unix 秒区间（约 2024–2100）。**无 timezone**（0.3.0 pin 的 `system-clock` 只有 `now` + `resolution`）。

```powershell
wasm-tools parse fixtures/wasi/system_now.wat -o fixtures/wasi/system_now.wasm
wasm-tools validate --features=component-model fixtures/wasi/system_now.wasm
```

## `wasi:clocks` — `monotonic-clock.resolution`

Guest export: `run: func() -> u64`（duration 纳秒）
Host: `wasi:clocks/monotonic-clock@0.3.0#resolution`（本机 `Instant` 按 1ns 粒度；钉 `@0.3.0`）

成功：返回 `1`。timezone / `system-clock.resolution` 另切片。

```powershell
wasm-tools parse fixtures/wasi/monotonic_resolution.wat -o fixtures/wasi/monotonic_resolution.wasm
```

## `wasi:clocks` — `system-clock.resolution`

Guest export: `run: func() -> u64`（`instant.nanoseconds`，要求 `seconds == 0`）  
Host: `wasi:clocks/system-clock@0.3.0#resolution`（`{seconds: 0, nanoseconds: 1}`；钉 `@0.3.0`）

成功：返回 `1`。无 timezone。

```powershell
wasm-tools parse fixtures/wasi/system_resolution.wat -o fixtures/wasi/system_resolution.wasm
wasm-tools validate --features=component-model fixtures/wasi/system_resolution.wasm
```

## `wasi:cli/command` — async `run`（子集）

Guest: 根导出 `run: async func() -> u32`（0 = ok，仪器 `callRunConcurrent`）  
官方导出：`wasi:cli/run@0.3.0#run: async func() -> result`（empty ok）  
Host: 复用已有 `wasi:cli/stdout@0.3.0#write-via-stream`

本切片是 **command-shaped** 子集：guest 写 `CMD\n` 后 official `result` 为 ok。**不是**完整 `wasi:cli/command` world（无 filesystem / sockets / environment / exit / terminal）。

成功：根 `run` 返回 `0`；`wasi:cli/run@0.3.0#run` 为 ok。

```powershell
wasm-tools parse fixtures/wasi/cli_command.wat -o fixtures/wasi/cli_command.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/cli_command.wasm
```

## `wasi:filesystem` — preopen + read/write（Android 沙箱子集）

Guest export: `run: func() -> u32`（写 `P3FS` 再读回，返回 4）  
Host: `wasi:filesystem/preopens@0.3.0#get-directories` → 沙箱**目录** `list`（名 `"."`）；`[method]descriptor.open-at("p3fs.txt")` → child；write/read-via-stream 带 `offset: filesize`（钉 `@0.3.0`）

官方包名如上。本切片：目录 preopen + `open-at` 成功路径；guest `open-at("..")` → `error-code.access`；write/read 取 `offset: filesize`（smoke 用 `0`）。沙箱见 [`docs/mapping/threading-android.md`](../../docs/mapping/threading-android.md) §5。G-fs-shape / G-fs-open **已完成**。

成功：guest `run` 返回 `4` 且宿主文件内容为 `P3FS`。

```powershell
wasm-tools parse fixtures/wasi/filesystem_preopen.wat -o fixtures/wasi/filesystem_preopen.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/filesystem_preopen.wasm
```

## `wasi:sockets` — TCP loopback echo（Android 子集）

Guest export: `run: async func() -> u32`（写 `P3SK`，经 loopback echo 读回，返回 4）  
Host: `wasi:sockets/tcp-create-socket@0.3.0#create-tcp-socket`；`[method]tcp-socket.connect`（钉 `@0.3.0`）

官方包名如上。本切片：`create-tcp-socket(ip-address-family) -> result`（smoke `ipv4`）；`connect: async func(ip-socket-address) -> result`（guest 传 loopback，host 可忽略 port，仍用 echo pair）；write/read 走 stream。无 UDP / listen / name-lookup。仅 `127.0.0.1`。Android 需要 **INTERNET**（含 loopback）；阻塞 IO 在 helper 线程，见 [`docs/mapping/threading-android.md`](../../docs/mapping/threading-android.md) §6。G-sock-shape **已完成**。

成功：guest `run` 经 `run_concurrent` 返回 `4`。

```powershell
wasm-tools parse fixtures/wasi/sockets_tcp.wat -o fixtures/wasi/sockets_tcp.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/sockets_tcp.wasm
```

## `wasi:sockets` — TCP outbound（非回环拨号）

Guest export: `run: async func() -> u32`（写 `P3SK`，经 **host 真拨** 的 peer echo 读回，返回 4）  
Host: 同上 `connect`；guest 地址在 mem `P3IP` 记录（port + ipv4），测试在 instantiate 前打补丁。

**P010-TCP：** 非回环 IPv4 时 host **dial 该地址**（不是 ignore-port + echo pair）。回环仍走 W7 echo pair。无 UDP / listen。沙箱：出站 + INTERNET；默认不 listen。

成功：guest `run` 返回 `4` **且** 测试侧 echo 服务器收到 `P3SK`。

```powershell
wasm-tools parse fixtures/wasi/sockets_tcp_out.wat -o fixtures/wasi/sockets_tcp_out.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/sockets_tcp_out.wasm
```

## `wasi:http` — incoming-handler（进程内 ABI 子集）

Guest export: 根 `run: async func() -> u32`（200）；官方 `wasi:http/incoming-handler@0.3.0#handle: async func(own<request>) -> result<own<response>, error-code>`  
Host: `wasi:http/types@0.3.0` constructors + `status-code`（钉 `@0.3.0`）

官方包名如上。本切片子集：handle 官方 `result`（ok 路径）；**P010-HBODY** 另见下节 body `stream<u8>`。无 fields / outparam。**不是**监听 HTTP 服务器。未加 `wasmtime-wasi`（体积 + Android 线程，见 changelog）。线程契约见 [`docs/mapping/threading-android.md`](../../docs/mapping/threading-android.md) §7。G-http-shape **已完成**。**P010-HCTOR：** 本夹具仍 import `[constructor]request`/`response`，只挂测试 linker（`Linker.createWithFixtureConstructors`）。产品路径见下节 `http_handle`。

成功：根 `run` 经 `run_concurrent` 返回 `200`；官方 `handle` 返回的 response `status-code` 为 `200`。

```powershell
wasm-tools parse fixtures/wasi/http_handler.wat -o fixtures/wasi/http_handler.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/http_handler.wasm
```

## `wasi:http` — product handle without constructors（P010-HCTOR）

Guest export: 根 `run: async func() -> u32`（200）；官方 `handle` 不 import `[constructor]request`/`response`。Guest 用 `[static]response.new` 造空 body 的 200 response。Host 调 `handle` 时 `table.push` request。

成功：产品 linker instantiate 本夹具；根 `run` 返回 `200`；host 提供 request 后 `handle` 的 response `status-code` 为 `200`。import 构造器的 `http_handler.wasm` 在产品 linker 上 instantiate 失败。

```powershell
wasm-tools parse fixtures/wasi/http_handle.wat -o fixtures/wasi/http_handle.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/http_handle.wasm
```

## `wasi:http` — body `stream<u8>`（P010-HBODY）

Guest export: `run: async func() -> u32`（读请求 body `HBOD`，经 `response.new` 写回，再 `consume-body` 读回，返回 4）  
Host: `wasi:http/types@0.3.0` `[static]request.consume-body` / `[static]response.new` / `[static]response.consume-body`（钉 `@0.3.0`）

官方 `consume-body` 还带 `res` future 与 `option<trailers>`；官方 `new` 还带 headers。本刀子集：`tuple<stream<u8>, future<result<_, error-code>>>`，无 trailers / headers。出站 send 见下节。

成功：guest `run` 返回 `4`。

```powershell
wasm-tools parse fixtures/wasi/http_body.wat -o fixtures/wasi/http_body.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/http_body.wasm
```

## `wasi:http` — outbound `client.send`（P010-HOUT）

Guest export: `run: async func() -> u32`（`set-authority` → `send` GET → status 200 → `consume-body` `HOUT` → 返回 4）  
Host: `wasi:http/client@0.3.0#send`（钉 `@0.3.0`；0.3 对 outgoing-handler 的等价物）

Guest authority 在 mem `P3HA` 记录（len + `host:port`），测试 instantiate 前打补丁。Host **真拨** 该地址发 HTTP/1.1 GET（helper 线程），不是进程内 200。无 TLS crate；https → `unknown`。

成功：guest `run` 返回 `4` **且** 测试侧 HTTP 服务器收到 `GET /`。

```powershell
wasm-tools parse fixtures/wasi/http_out.wat -o fixtures/wasi/http_out.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/http_out.wasm
```

## `wasi-gfx` — `surface.on-frame`（P010-GFXH）

Guest export: `run: async func() -> u32`（构造 surface → `on-frame` → `stream.read` 一条 `frame-event` → 返回 1）  
Host: `wasi-gfx:surface/surface@0.2.0` constructor + `[method]surface.on-frame`（钉 tag `v0.2.0`）

Guest **拉** stream。Host 在名为 `GpuThread` 的 helper 线程上产生 vsync 载荷，再交给 `StreamReader`。钉里 `on-frame` 是同步 `func`（不是 `async func`）；本刀未开 Wasmtime stackful CM async。无 JS callback。上屏循环见下节 `gfx_frame_loop`（P010-GFXL）。

成功：guest `run` 返回 `1` **且** 事件在名为 `GpuThread` 的线程上产生。

```powershell
wasm-tools parse fixtures/wasi/gfx_on_frame.wat -o fixtures/wasi/gfx_on_frame.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/gfx_on_frame.wasm
```

## `wasi-gfx` — product frame loop（P010-GFXL）

Guest export: `run: async func() -> u32`（surface + `surface-webgpu` context → configure → `on-frame` 循环：`get-current-texture` → `queue.submit` → `context.present` → 返回帧数 2）  
Host: `wasi-gfx:surface/surface@0.2.0` + `wasi-gfx:surface/surface-webgpu@0.2.0`（钉 tag `v0.2.0`）

GPU 起步仍用 fixture `get-device`（P010-FIX）。`get-queue` / `get-command-buffer` 为产品 leftover。两帧事件在 `GpuThread` 上预缓冲（无 stackful）。无 JS callback。WG-6 单帧回归保留。设备仪器：`WasiGfxFrameLoopInstrumentedTest`。

```powershell
wasm-tools parse fixtures/wasi/gfx_frame_loop.wat -o fixtures/wasi/gfx_frame_loop.wasm
wasm-tools validate --features=cm-async,component-model fixtures/wasi/gfx_frame_loop.wasm
```
