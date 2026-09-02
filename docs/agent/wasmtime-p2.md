# Agent playbook: Wasmtime pin (P2)

**English** | [中文](wasmtime-p2.zh.md)

**Named-only.** Living leftover is [`remaining.md`](remaining.md). Use this playbook only when the user names P2 / Wasmtime pin.

Tracking: [`../scheme/wasmtime-tracking.md`](../scheme/wasmtime-tracking.md). Current pin: **47.0.4** (`native/Cargo.toml`). Dependabot ignores **major**.

## Goal

A third party can read the tracking table and know: pin, last-checked date, upstream latest at check, and the next allowed upgrade path (patch vs RFC). Not a `wasmtime-wasi` crate. Not a WASI re-cut.

## Select the cut

```text
python3 ./scripts/wasmtime-p2-remaining.py
```

Do the printed **Next:** only.

## Hard bans

- Do **not** add `wasmtime-wasi` unless that PR’s changelog records a size + Android thread review.
- Do **not** introduce wasmtime4j.
- Do **not** land a **major** (47 → 48+) without a short upgrade RFC.
- Do **not** treat Latch / sync-compat as true CM async (NG-8).
- Never file upstream GitHub issues.

## Named leftovers (not this script)

G-cmd, G-fs-full, listen/UDP: [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md).
