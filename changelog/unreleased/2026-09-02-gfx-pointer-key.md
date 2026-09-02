### Code — wasi-gfx pointer/key true events (2026-09-02)

- `Store.postGfxPointer` / `postGfxKey` post into bounded host queues. Product linker `on-pointer-*` / `on-key-*` streams wait on those gates (sync `func`; no `Poll::Pending`).
- Android `KeyEvent.keyCode` maps onto pin `key` when known; otherwise `key: none` with optional `text`.
- `gfx_pin` still opens and drops unread streams. `gfx_input` reads one posted pointer-down.
