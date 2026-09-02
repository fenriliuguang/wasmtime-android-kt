# Cube hitch restart queue (tracking)

**English** | [中文](gfx-hitch.zh.md)

Living **auto** leftover after native-dawn consume: bind the NativeGpu cube **eye pop** to a hot-path stage (or to no in-process spike). Playbook: [`../agent/gfx-hitch.md`](../agent/gfx-hitch.md). Beat map: [`../mapping/gfx-hitch-native-dawn.md`](../mapping/gfx-hitch-native-dawn.md) §6.

Not a consume needle. Do **not** inherit mapping §§0–5 Closed/Likely as premises. Do **not** re-cut P0 / P1 / `0.1.0` / `ND-*`.

Branch: **`fix/300-gfx-cube-pop`**. Remaining: `python3 ./scripts/gfx-hitch-remaining.py` (next **commit**, not next PR). A lane drops when its **`gap: hitch … pending`** needle leaves **this file**.

## Needles (auto order)

<!-- remaining.py greps these exact strings. Keep one per unfinished lane. -->

| Lane | Needle (delete when landed) |
|------|-----------------------------|
| HP-RFC | landed 2026-09-02 (playbook / skill / remaining; forget inherited localization) |
| HP-LOG | landed 2026-09-02 (V2458A 150 s `hotpath` / `hotpath-spike`; mapping §6.6) |
| HP-BIND | landed 2026-09-02 (no ~5 s in-process stage spike; 45 s S3b acquire burst) |

## Named-only (never `Next:`)

After HP-BIND: compositor/acquire **SF counters landed 2026-09-02** (mapping §6.7; no knob). Remaining named: guest/CM encode-gap, GPU fence vs D24 `onSubmittedWorkDone`, event-triggered `screenrecord` (next after §6.7). Banned: keep-N, DisplayManager, GameState, JNI removal. P2 Wasmtime, CTS, this-repo **1.0.0**.
