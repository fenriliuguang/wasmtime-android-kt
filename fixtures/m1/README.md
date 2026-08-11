# M1 sync CM fixture

Minimal Component Model guest with **no imports**:

- export `run: func(a: u32) -> u32` → returns `a + 1`

```text
add_one.wat   source (wasm-tools parse)
add_one.wasm  checked-in binary for instruments / offline builds
```

Regenerate:

```powershell
wasm-tools parse fixtures/m1/add_one.wat -o fixtures/m1/add_one.wasm
wasm-tools validate fixtures/m1/add_one.wasm
```
