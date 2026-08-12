# W1 wasi:webgpu request-adapter fixture

| File | Imports | Export | Behavior |
|------|---------|--------|----------|
| `webgpu_request_adapter.wasm` | `wasi:webgpu/webgpu@0.3.0-rc.2#request-adapter` → `u32` | `run: func() -> u32` | returns adapter rep from same L2 as M3 |

**Transitional:** host registers flat `request-adapter` (not `[method]gpu.request-adapter`). Sync-compat u32 only — not proposal `async func` (W2).

Regenerate:

```powershell
wasm-tools parse fixtures/w1/webgpu_request_adapter.wat -o fixtures/w1/webgpu_request_adapter.wasm
wasm-tools validate fixtures/w1/webgpu_request_adapter.wasm
```
