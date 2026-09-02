---
name: gfx-hitch
description: >-
  Cube ~5 s eye-pop closed as out-of-tree guest sincos (mapping §6.9).
  Use when the user says 抖动, hitch, cube-pop, issue 300, gfx-hitch,
  真机, HP-LOG, HP-BIND, or run gfx-hitch-remaining.
---

# Cube hitch restart (hot-path stages)

Read [`docs/agent/gfx-hitch.md`](docs/agent/gfx-hitch.md). Beat map: [`docs/mapping/gfx-hitch-native-dawn.md`](docs/mapping/gfx-hitch-native-dawn.md) **§6.9**.

1. Work on **`fix/300-gfx-cube-pop`**. `python3 ./scripts/gfx-hitch-remaining.py` prints **empty**. Do **not** open a per-lane PR.
2. Eye-pop is **guest Taylor/`wrap_pi`** (then shared-Euler `fold_pi`). Not Dawn C, not compositor, not keep-N.
3. Do **not** restack keep-N / DisplayManager / GameState / JNI / Mailbox / fence for this pop.
4. Native-dawn consume leftover is empty. Never file upstream GitHub issues.
