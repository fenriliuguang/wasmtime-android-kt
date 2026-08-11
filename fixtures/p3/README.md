# P3 stream read smoke

Guest export: `read: func(s: stream<u8>, l: u32) -> u32`  
Host: `StreamReader::new(store, vec![b'P', b'3', b'S', b'T'])` then call `read` with `l = 100`.

Expected packed result: `(4 << 4) | 1` = **65** (4 bytes + DROPPED).

Build:

```powershell
wasm-tools parse fixtures/p3/stream_read.wat -o fixtures/p3/stream_read.wasm
```
