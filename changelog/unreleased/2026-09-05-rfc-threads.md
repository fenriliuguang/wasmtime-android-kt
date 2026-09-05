### Docs — Draft RFC: guest concurrency / threads (2026-09-05)

- Add [`docs/scheme/rfc-threads.md`](../../docs/scheme/rfc-threads.md) (**Draft**, no implementation): host thread contract already shipped; the gap is Wasmtime **stackful** CM async, not `wasi:threads` / shared-everything
- Strawman is option B (enable stackful on Android) after ART / GpuThread / hitch / size questions; option D (guest wasm threads) is reject
- Do not amend accepted [`rfc.md`](../../docs/scheme/rfc.md) until this RFC is accepted
