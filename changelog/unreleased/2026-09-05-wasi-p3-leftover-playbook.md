### Docs — start WASI 0.3 leftover long branch (2026-09-05)

- Living leftover queue after `0.1.2`: thin-host fill of G-err / G-cmd / G-fs-full / G-sock-rest / G-http on **`cursor/wasi-p3-leftover-b677`** (one lane = one commit; one PR when remaining is empty)
- Playbook [`docs/scheme/wasi-p3-leftover.md`](../../docs/scheme/wasi-p3-leftover.md); remaining `python3 ./scripts/wasi-p3-leftover-remaining.py` (first **Next:** `L-ERR-CLI`)
- NG-4 stays (not wasi-testsuite). Do not add `wasmtime-wasi`. Guest concurrency is a separate Draft RFC. Benches stay deferred
