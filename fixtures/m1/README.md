# M1 sync CM fixtures

| File | Imports | Export | Behavior |
|------|---------|--------|----------|
| `add_one.wasm` | — | `run(a) -> u32` | `a + 1` |
| `host_add.wasm` | `add(a,b)->u32` | `run(a,b)->u32` | calls host `add` |
| `widget_echo.wasm` | `widget` resource, `make-widget`, `echo-widget` | `run(rep)->u32` | make then echo u32 rep |

Regenerate:

```powershell
wasm-tools parse fixtures/m1/add_one.wat -o fixtures/m1/add_one.wasm
wasm-tools parse fixtures/m1/host_add.wat -o fixtures/m1/host_add.wasm
wasm-tools parse fixtures/m1/widget_echo.wat -o fixtures/m1/widget_echo.wasm
```
