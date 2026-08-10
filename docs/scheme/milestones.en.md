# Milestones & DoD (Track B)

[中文](milestones.md) | **English**

Order: **M0 → M1 → M2 (hard gate) → M3 → M4 → M5**.  
If M2 fails, stop M3/M4 L2/graphics spend until runtime is fixed.  
**This docs init completes none of the code DoDs.**

| ID | Name | One-line DoD |
|----|------|----------------|
| M0 | Skeleton | ART loads our Wasmtime `.so`; reproducible build docs |
| M1 | Sync CM | Min host import + guest export round-trip; u32 resource rep |
| M2 | True CM async | Host create/complete/reject future; guest observes; threading note |
| M3 | Plug L2 | ≥1 adapter-path call via this L1 → Track A L2; A cube CI still green |
| M4 | On-screen smoke | Cube subset or dedicated guest on Dawn; separate instrumented path |
| M5 | Harden | Errors, multi-ABI layout, API policy, contributor docs |

Status: docs charter **done** (2026-08-10); M0–M5 **not started**.
