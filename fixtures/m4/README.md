# M4 dedicated render smoke fixture

| Artifact | Host imports (flat u32) | Export | Behavior |
|----------|-------------------------|--------|----------|
| `render_smoke.wasm` | clear→present subset on `experimental:webgpu-cm/host@0.8.0` | `run-clear: func(u64,u32,u32)->u32` | clear color + present; returns `0` |

Handles are **u32 reps** (M3 style), not WIT `adapter`/`device` resources. Product shape: [`docs/scheme/guest-shape.md`](../../docs/scheme/guest-shape.md).

```powershell
wasm-tools component new fixtures/m4/render_smoke.wat -o fixtures/m4/render_smoke.wasm
# or, for text components:
wasm-tools parse fixtures/m4/render_smoke.wat -o fixtures/m4/render_smoke.wasm
```
