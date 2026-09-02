# Remaining close-out (tracking)

**English** | [中文](remaining.zh.md)

Living **auto** queue after the native Dawn host landed. Playbook: [`../agent/remaining.md`](../agent/remaining.md). Policy: [`rfc.md`](rfc.md).

Needles stay in **this file**. `python3 ./scripts/remaining.py` prints the next **one** lane. Do not remove a needle without that lane’s DoD.

<!-- remaining.py greps these exact strings. Keep one per unfinished lane. -->

Auto queue is **empty**.

## Named-only (never `Next:`)

`context.unconfigure`; timestamped `frame-event`; Lost/Outdated as `result`; multi-window. Also: P2 Wasmtime pin, G-cmd, G-fs-full, listen/UDP, wasi-testsuite, `wasmtime-wasi`, this-repo **1.0.0**, CTS. Never file upstream issues.
