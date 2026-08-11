# M3 experimental request-adapter fixture

| File | Imports | Export | Behavior |
|------|---------|--------|----------|
| `request_adapter.wasm` | `experimental:webgpu-cm/host@0.8.0#request-adapter` → `u32` | `run: func() -> u32` | returns adapter rep from L2 |

M3 uses a **u32** result (rep) rather than a WIT `adapter` resource so the first L1→L2 slice stays small. Resource-typed adapter lands with fuller CM world wiring.

Regenerate:

```powershell
wasm-tools parse fixtures/m3/request_adapter.wat -o fixtures/m3/request_adapter.wasm
wasm-tools validate fixtures/m3/request_adapter.wasm
```
