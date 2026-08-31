# Native Dawn host queue (tracking)

**English** | [中文](native-dawn.zh.md)

Living **auto** queue: default `wasi:webgpu` consume via **Dawn C** on the Wasmtime pump thread. Playbook: [`../agent/native-dawn.md`](../agent/native-dawn.md). Guest pin unchanged (`0.3.0-rc.2`). `0.1.0` gates: [`product-010.md`](product-010.md) (empty).

P0 / P1 auto knives stay **closed**. P2 Wasmtime pin is **named**. Do **not** re-cut G1–G9 / F1–F9 / WG-6 as those queues.

Branch: **`cursor/native-dawn-rewrite-1355`**. Remaining: `python3 ./scripts/native-dawn-remaining.py` (next **commit**, not next PR). A lane drops when its **`gap: nd … pending`** needle leaves **this file**. Do **not** remove a needle without landing that lane’s DoD. Open **one** PR to `main` only when this table has no pending needles.

Cube / out-of-tree demo is **not** a consume needle.

## Needles (auto order)

<!-- remaining.py greps these exact strings. Keep one per unfinished lane. -->

| Lane | Needle (delete when landed) |
|------|-----------------------------|
| ND-RFC | landed 2026-08-31 (playbook / skill / gates) |
| ND-DISP | landed 2026-08-31 (NativeGpu \| JniBackend; JNI default) |
| ND-SO | landed 2026-08-31 (Dawn C API Android `.so` recipe; JNI leftover still default) |
| ND-HOST | landed 2026-08-31 (`NativeGpu` trait + handle table; no product Kotlin GPU API) |
| ND-BOOT | landed 2026-08-31 (native request-adapter / request-device / queue + boot info; table-backed) |
| ND-RES | `gap: nd res pending` |
| ND-PIPE | `gap: nd pipe pending` |
| ND-ENC | `gap: nd enc pending` |
| ND-QUEUE | `gap: nd queue pending` |
| ND-REST | `gap: nd rest pending` |
| ND-SURF | `gap: nd surf pending` |
| ND-DEFAULT | `gap: nd default pending` |
| ND-CLAIM | `gap: nd claim pending` |
| ND-DEVICE | `gap: nd device pending` |

## Named-only (never `Next:`)

Cube-only hot path, hitch D3 present-timestamp, `AChoreographer`, core pin, G-cmd, testsuite, `wasmtime-wasi`, CTS, this-repo **1.0.0**, P0/P1 re-cuts, Wasmtime **major**.
