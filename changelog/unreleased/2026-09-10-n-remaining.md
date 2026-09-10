### Docs — NativeGpu Remaining Table auto queue (2026-09-10)

- Playbook [`docs/scheme/nativegpu-remaining.md`](../../docs/scheme/nativegpu-remaining.md); `python3 ./scripts/nativegpu-remaining.py`. WASI leftover empty is **not** “nothing to cut”.
- Catch-up knives with this amendment: **N-LIMITS** (`GetLimits` when `.so` loads; Cloud still `1`), **N-COPY** (origin / mip / aspect / layout), **N-DYNOFF** (`set-bind-group` offsets). Next auto: **N-COMPINFO**.
