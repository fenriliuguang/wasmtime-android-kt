### Docs — P0 close-out + P1 WASI 0.3 playbook (2026-08-22)

- Archive P0 implementation playbooks/skills (`webgpu-guest-pipeline` / `semantics` / `dawn`); add compressed close-out [`docs/archive/p0-wasi-webgpu.md`](../../docs/archive/p0-wasi-webgpu.md) and WIT ↔ androidx gap [`docs/mapping/gap-webgpu-wit-androidx.md`](../../docs/mapping/gap-webgpu-wit-androidx.md)
- Living queue is P1: [`docs/agent/wasi-p3.md`](../../docs/agent/wasi-p3.md), skill `wasi-p3`, `.\scripts\wasi-p3-remaining.ps1` (W1 stream multi-chunk → W8 http; device instrument per lane)
- Do not re-cut G1–G9 / F1–F9 / WG-6; do not add `wasmtime-wasi` without a size + thread review
