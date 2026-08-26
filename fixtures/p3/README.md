# P3 stream smokes

## Read（host → guest）

Guest export: `read: func(s: stream<u8>, l: u32) -> u32`  
Host: `StreamReader::new(store, vec![b'P', b'3', b'S', b'T'])` then call `read` with `l = 100`.

Expected packed result: `(4 << 4) | 1` = **65** (4 bytes + DROPPED).

## Write（guest → host，写方向翻转）

Guest export: `run: func() -> u32`  
Host import: `take: func(s: stream<u8>) -> future<u32>`（`pipe` + `StreamConsumer`）  
Guest: `stream.new` → `take(readable)` → `stream.write` `P3WR` → `drop-writable` → `future.read`.

Expected: **4**（consumed byte count）.

## Multi-chunk + backpressure（W1）

Guest export: `run: func() -> u32`  
Host import: `take-chunks: func(s: stream<u8>) -> future<u32>`（`pipe` + 2-byte-per-poll `StreamConsumer`）  
Guest: `stream.new` → `take-chunks(readable)` → three `stream.write` of `P3C1` / `P3C2` / `P3C3` (retry remaining on partial count) → `drop-writable` → `future.read`.

Expected: **12**. Not a second copy of the 4-byte `P3ST` / `P3WR` smokes.

## Build

```powershell
wasm-tools parse fixtures/p3/stream_read.wat -o fixtures/p3/stream_read.wasm
wasm-tools parse fixtures/p3/stream_write.wat -o fixtures/p3/stream_write.wasm
wasm-tools parse fixtures/p3/stream_chunks.wat -o fixtures/p3/stream_chunks.wasm
wasm-tools validate --features=cm-async,component-model fixtures/p3/stream_chunks.wasm
```
