# Agent playbook: cube hitch restart (hot-path stages)

**English** | [中文](gfx-hitch.zh.md)

P0 `wasi:webgpu` **shape** is **closed**. P1 WASI 0.3 official-shape is **closed**. `0.1.0` product gates are **empty**. Native Dawn **consume** leftover is **empty** (`ND-DEVICE` landed; `#299` on `main`). Do **not** re-cut those queues. Do **not** treat this hitch as a consume needle.

Living **auto** leftover after consume: **empty** (eye-pop bound to out-of-tree guest `sincos`; mapping §6.9). Tracking: [`../scheme/gfx-hitch.md`](../scheme/gfx-hitch.md). Beat map: [`../mapping/gfx-hitch-native-dawn.md`](../mapping/gfx-hitch-native-dawn.md) **§6**.

**Restart rule.** Mapping §§0–5 Closed / Likely / Mitigated rows are **archive**. This queue does **not** inherit them as premises. Issue 300 on this branch already chose that: forget previous localization; start from the vsync→present beat in code.

**Integration.** Branch: **`fix/300-gfx-cube-pop`**. One lane = **one commit**. Stay on it. Do **not** open a per-lane PR. Open or update the issue-300 PR when remaining is empty (or when the user asks).

P2 Wasmtime pin stays **named-only** ([`wasmtime-p2.md`](wasmtime-p2.md)). Consume playbook: [`native-dawn.md`](native-dawn.md).

## Goal

On Vivo V2458A (Android 16, Settings lock **120 Hz**), the out-of-tree rotating cube’s ~5 s **eye pop** is **closed** as guest Taylor/`wrap_pi` (then a shared-Euler `fold_pi` snap). NativeGpu / Dawn C present path was not the cause. Control androidx cube never popped. Not CTS. Not a pin-consume DoD.

## Why this restart (commits on this branch)

| Commit | What it did | Use as |
|--------|-------------|--------|
| `ad64463` | NativeGpu cube path; D25 pop remains after dropping androidx JNI | Archive: JNI removal is not this queue |
| `b9aeb3d` | Present timestamps; P2–P5 / N9 probes closed guest / CM / SF counters / submit | Archive: do not recut those probes as premises |
| `94d5983` | **Restart:** forget inherited Closed/Likely; map the beat; log per-stage `Instant` | Current plan (mapping §6) |
| `9da8763` | Cloud synthetic 120 Hz 1:1 beats | Cloud can check the beat machine; **does not** close the eye-pop |

Do **not** reopen D2/D3/N4 histograms as the next auto cut. Those log lines may still print; HP-LOG reads **`hotpath` / `hotpath-spike`**.

## Select the cut

If the user named a lane (`HP-LOG`, `HP-BIND`, hitch, 抖动, cube-pop, issue 300), keep **one** family. Otherwise:

```powershell
.\scripts\gfx-hitch-remaining.ps1
```

No `pwsh`: `python3 ./scripts/gfx-hitch-remaining.py` (same flags: `--all`).

Do the printed **Next:** line only — as **one commit** on `fix/300-gfx-cube-pop`. `native-dawn-remaining` is empty; do not invent new `ND-*` needles.

## Hard bans

- Do **not** inherit Closed / Likely / Mitigated from [`gfx-hitch-checklist.md`](../mapping/gfx-hitch-checklist.md) or mapping §§0–5 as the reason to skip a stage.
- Do **not** restack keep-N, DisplayManager, GameState, SurfaceControl votes, or another JNI removal until HP-BIND names a stage (or “no in-process spike”).
- Do **not** treat Cloud `hotpath_synthetic_120hz_beats_are_1_to_1` as a device pass.
- Do **not** vendor the cube demo. Do **not** file upstream GitHub issues. No `gh issue create`.
- Do **not** add Mailbox as default. Do **not** JS-style frame callbacks.
- Do **not** recut native-dawn consume lanes or P0/P1 auto knives.
- Do **not** edit hub files on a lane commit: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`. Exception: this playbook / gate-amendment **commit**.
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` this slice changed.
- Never file GitHub issues on Wasmtime, WASI, Dawn, or androidx.

## Lanes (auto)

Copy this table. Do not close HP-LOG with a Linux histogram.

| Commit | Needle in [`../scheme/gfx-hitch.md`](../scheme/gfx-hitch.md) | DoD |
|--------|--------------------------------------------------------------|-----|
| **HP-RFC** | *(landed this playbook)* | Playbook + remaining script + skill/rule + native-dawn redirect. `gfx-hitch-remaining.py` prints **`Next: HP-LOG`**. |
| **HP-LOG** | `gap: hitch log pending` | Device: V2458A, Settings 120 Hz lock. Run the NativeGpu cube ≥2 min. Capture logcat `GfxHitch` `hotpath` (every 120 presents) and `hotpath-spike`. Record the window in mapping §6.6 (counts, max/last per stage, spike lines). Note whether the **eye** popped in that window. Do **not** conclude from archive Closed rows. Changelog. Remove the needle. |
| **HP-BIND** | `gap: hitch bind pending` | One sentence in mapping §6.6: bind the eye-pop to a **stage spike** **or** to **“no in-process spike at the pop.”** That sentence is the only auto DoD. Changelog. Remove the needle. Then stop unless the user names a follow-up. |

## Named-only (never `Next:`)

After HP-BIND, one variable. Mapping §6.3 order, not a remaining needle:

| Lane | When | DoD |
|------|------|-----|
| Compositor / acquire | Bind named S3 / S6b spike | One knob (BLAST / timestamp / Fifo). Not keep-N. **Landed 2026-09-02:** mapping §6.7 SF/gfxinfo re-measure; **no knob** (counters clean). |
| Guest / CM | Bind named S4 encode-gap spike | One knob on guest WIT or pump. **Not this eye-pop** (§6.9). |
| GPU fence vs D24 | Bind named S6a, or no CPU spike while the eye pops | Real `onSubmittedWorkDone`. **Not this eye-pop** (§6.9). |
| Event `screenrecord` | Bind named no in-process spike | Frames around the pop. **Superseded §6.9** (guest trig). |
| Guest sincos | Observer: no pop after Cody–Waite `sincos_d` + no shared-`θ` fold | Mapping §6.9. **Landed 2026-09-02.** |
| keep-N / DisplayManager / GameState / JNI | Banned until bind | Do not restack. This eye-pop does not un-ban them. |

## File whitelist (typical)

- `docs/mapping/gfx-hitch-native-dawn.md` / `.zh.md` — §6 / §6.6–§6.9
- `docs/scheme/gfx-hitch.md` — **remove this lane’s needle**
- `changelog/unreleased/<yyyy-mm-dd>-gfx-hitch-<slug>.md`
- `native/src/native_gpu.rs` — only if HP-LOG needs a log-format fix (not a present-path behavior change)

Do not add files under `docs/archive/`. Do not recut G1–G9 fixture names.

## Narrow tests

This playbook amendment (docs-only):

```powershell
python3 ./scripts/gfx-hitch-remaining.py
```

Must print **`Next: (gfx hitch restart queue empty)`** and name branch `fix/300-gfx-cube-pop`.

HP-LOG is **device**. Cloud cannot simulate Mali / BLAST / SurfaceFlinger. Do not close HP-LOG without the logcat window. Existing Cloud check (already landed): `cd native && cargo test --locked --lib hotpath_synthetic_120hz_beats_are_1_to_1`.

Device capture:

```powershell
adb logcat -s GfxHitch:I GfxHitch:W
```

Host: out-of-tree `hosts/fullscreen-surface`. Guest: same MoonBit cube. Settings: `min_refresh_rate=120` + vendor 120 Hz lock (already used on this device; do not add app votes).

## Commit message (one per lane)

- Workflow / this playbook: `docs: gfx hitch restart from hot-path stages`
- HP-LOG: `test(gfx): HP device hotpath window`
- HP-BIND: `docs: HP bind cube pop to hotpath stage`

- Close-out: `docs: HP bind cube pop to guest sincos`

## Copy source

Mapping §6 beat sequence and stage table. `NativeGpuHost` `finish_hotpath` log lines. Hitch recycle invariants (keep-3, Fifo, H8) stay **code facts**, not Closed conclusions. Do not copy C7 AAR leak batching.
