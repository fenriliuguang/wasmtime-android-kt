# Milestones & DoD (Track B)

[中文](milestones.md) | **English**

Order: **M0 → M1 → M2 (hard gate) → M3 → M4 → M5**.  
If M2 fails, stop M3/M4 L2/graphics spend until runtime is fixed.  
Authoritative checkboxes: [`milestones.md`](milestones.md) (ZH).

| ID | Name | One-line DoD | Status |
|----|------|----------------|--------|
| M0 | Skeleton | ART loads our Wasmtime `.so`; reproducible build docs | **done** |
| M1 | Sync CM | Min host import + guest export round-trip; u32 resource rep | **done** |
| M2 | True CM async | Host create/complete/reject future; guest observes; threading note | **done** |
| M3 | Plug L2 | ≥1 adapter-path call via this L1 → Track A L2; A cube CI still green | **done** |
| M4 | On-screen smoke | Dedicated Dawn clear→present guest; separate instrumented path | **done** |
| M5 | Harden | Errors, multi-ABI, API policy, contributor/desktop shell, WASI worlds RFC | **done** (2026-08-11) |
