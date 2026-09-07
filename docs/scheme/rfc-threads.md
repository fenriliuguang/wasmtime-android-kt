# RFC: Guest concurrency / “threads”

**Status: Draft** · 2026-09-05 · design discussion only (no implementation on this branch)

English is canonical. Short Chinese: [`rfc-threads.zh.md`](rfc-threads.zh.md).

Accepted product policy stays in [`rfc.md`](rfc.md). This file is **not** accepted until it is merged into that RFC or withdrawn. Sibling: leftover fill [`rfc-wasi-p3.md`](rfc-wasi-p3.md) (Draft). Out-of-tree examples and benches are out of scope.

## 1. Problem

“Support threads” is overloaded. The host already has a thread contract ([`../mapping/threading-android.md`](../mapping/threading-android.md)). Guests still cannot suspend on a `BLOCKED` CM `stream.read`: pin `wasi-gfx` `on-frame` is a **sync** `func`, and this repo does **not** enable Wasmtime **stackful** CM async (guest WAT traps). That is the real concurrency gap. It is **not** `wasi:threads` / shared-everything / wasm pthread.

## 2. What already shipped (do not re-cut)

| Piece | Contract |
|-------|---------|
| GpuThread | Dawn + `processEvents` + present |
| One `run_concurrent` driver per Store | Never two drivers on the same Store |
| Helper threads | sockets connect, HTTP `send`, `clocks.wait-for` / `wait-until` |
| `wasmtime-cm-pump` | 8 MiB pthread; ART ~1 MiB stacks overflow; do not `AttachCurrentThread` on that custom stack |
| gfx pull-stream | 1-slot vsync gate; unconsumed beats drop |

Do **not** replace GpuThread, add JS-style `start(callback)`, or treat Latch/sync-compat as true CM async (NG-8).

## 3. Options

| ID | Proposal | Notes |
|----|----------|-------|
| **A** | Status quo | Document the table in §2 as the product thread story. Guest still cannot `Poll::Pending` on `on-frame` produce. |
| **B** | Enable Wasmtime **stackful** CM async on Android | Guest `stream.read` may block; producer may `Pending`. Touches ART attach, hitch, GpuThread exclusivity, `.so` size. |
| **C** | Extra Stores / extra CM pumps | Already legal if Stores are distinct. Does **not** give one guest concurrent tasks. |
| **D** | Guest wasm threads / shared memory / `wasi:threads` | **Reject** for this RFC. Not WASI 0.3; fights GpuThread + hitch. |

**Strawman (not accepted):** discuss **B**. Keep A until this RFC is accepted. Do not land B on a leftover WASI commit.

## 4. Questions B must answer before code

1. **Pump thread:** stay on 8 MiB `wasmtime-cm-pump`, or a stackful-safe ART thread? JNI bounce rules stay.
2. **GpuThread:** if a guest suspends mid-frame, who still owns Dawn / `ANativeWindow`? One driver per Store still holds.
3. **Hitch:** `on-frame` is sync in the pin. Stackful must not reintroduce mid-frame vsync take, acquire-wait of the previous fence, or Mailbox.
4. **Size:** changelog must record `.so` / feature cost (same bar as `wasmtime-wasi`).
5. **Proof:** a WAT that `stream.read`s `BLOCKED` then resumes; gfx instruments stay green. Cloud cannot simulate Mali present.

## 5. Out of this RFC

- Filling WASI 0.3 named leftovers ([`rfc-wasi-p3.md`](rfc-wasi-p3.md), long branch `cursor/wasi-p3-leftover-b677`)
- `wasmtime-wasi`
- Wasmtime **major** (P2 named)
- Benchmarks (deferred)
- Out-of-tree examples (separate repo)

## 6. Acceptance when this RFC is accepted

- Amend [`rfc.md`](rfc.md) §3 / [`threading-android.md`](../mapping/threading-android.md) with the chosen option (A or B).
- Option B becomes a **named** implementation queue (not leftover `L-*` lanes). Do not auto-cut it from the leftover remaining script.
- Still never file upstream GitHub issues.
