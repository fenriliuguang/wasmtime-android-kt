# M2 true CM async fixture

| File | Imports | Export | Behavior |
|------|---------|--------|----------|
| `async_get.wasm` | `get: async func() -> u32` | `run: async func() -> u32` | sync-lower `get` inside async export; returns host value |

Host (native): `func_wrap_concurrent("get")` creates a `FutureReader` (oneshot producer), **completes** it with `42`, then returns `42` to the guest. Call path uses `pollster::block_on(store.run_concurrent(... call_concurrent ...))`.

Regenerate:

```powershell
wasm-tools parse fixtures/m2/async_get.wat -o fixtures/m2/async_get.wasm
wasm-tools validate --features=cm-async,component-model fixtures/m2/async_get.wasm
```
