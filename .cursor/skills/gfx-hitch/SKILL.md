---
name: gfx-hitch
description: >-
  After native-dawn consume: restart cube ~5 s eye-pop from hot-path stages
  (forget inherited Closed/Likely). Device GfxHitch hotpath / hotpath-spike,
  then bind the pop to a stage or to no in-process spike. Use when the user
  says 抖动, hitch, cube-pop, issue 300, gfx-hitch, 真机, HP-LOG, HP-BIND,
  or run gfx-hitch-remaining.
---

# Cube hitch restart (hot-path stages)

Read and follow [`docs/agent/gfx-hitch.md`](docs/agent/gfx-hitch.md) before exploring. Beat map: [`docs/mapping/gfx-hitch-native-dawn.md`](docs/mapping/gfx-hitch-native-dawn.md) **§6**.

1. Work on **`fix/300-gfx-cube-pop`**. Run `.\scripts\gfx-hitch-remaining.ps1` (or `python3 ./scripts/gfx-hitch-remaining.py`) unless the user named one lane. Do **only** the printed **Next:** as **one commit**. **Do not open a per-lane PR.**
2. **Forget** mapping §§0–5 Closed / Likely / Mitigated as premises. Do **not** recut D2/D3/N4 histograms. HP-LOG reads `hotpath` / `hotpath-spike`.
3. Order: **HP-LOG** (device ≥2 min) → **HP-BIND** (one bind sentence) → named-only follow-up. Cloud synthetic 120 Hz does **not** close the eye-pop.
4. Until HP-BIND names a stage: do **not** restack keep-N / DisplayManager / GameState / JNI removal.
5. Native-dawn consume leftover is empty: [`docs/agent/native-dawn.md`](docs/agent/native-dawn.md). Do **not** invent `ND-*` needles. Never file upstream GitHub issues.
6. Hub freeze on lane commits: no root README / `CHANGELOG.md` / `ci.yml` / `CONTRIBUTING.md` except this playbook / gate-amendment **commit**.
7. Commit message from the playbook.
