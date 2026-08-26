# Agent playbook: Wasmtime pin (P2)

**English** | [中文](wasmtime-p2.zh.md)

P0 `wasi:webgpu` is **closed** ([`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md)). P1 WASI 0.3 official-shape is **closed** ([`../archive/p1-wasi-p3.md`](../archive/p1-wasi-p3.md)). Do **not** re-cut W1–W8, P1-FS1–FS4, P1-SK1–SK2, P1-HT1, G-dev, or wasi:webgpu G1–G9 / F1–F9 / guest-pipeline / WG-6.

This queue keeps the **upstream `wasmtime` pin knowable, upgradeable, and rollback-able**. Tracking table: [`../scheme/wasmtime-tracking.md`](../scheme/wasmtime-tracking.md). KPI is **not** “always on latest major”. One lane, one PR.

P1 leftover WIT shapes (G-err, G-cmd, G-fs-full, G-sock-rest, G-http-body, G-http-ctor, G-cli-error) live in [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) as **named future optimizations**. They are **not** this script’s `Next:`.

## Goal

A third party can read §2 / §3 of the tracking table and know: pin, last-checked date, upstream latest at check, and the next allowed upgrade path (patch vs RFC). Not a `wasmtime-wasi` crate. Not a WASI 0.3 re-cut. Not Maven Central.

Current pin: **47.0.4** (`native/Cargo.toml`). Dependabot ignores **major**.

## Select the cut

If the user named a lane, keep **one** family. Otherwise:

```powershell
.\scripts\wasmtime-p2-remaining.ps1
```

No `pwsh`: `python3 ./scripts/wasmtime-p2-remaining.py` (same flags: `--all`).

Do the printed **Next:** line only.

## Hard bans

- Do **not** re-cut P0 wasi:webgpu or P1 WASI 0.3 auto knives. GPU and WASI leftover pages are documentation / named-only.
- Do **not** add `wasmtime-wasi` as a Cargo dependency unless that PR’s changelog records a size + Android thread review.
- Do **not** introduce `ai.tegmentum:wasmtime4j` or 4j native as the runtime.
- Do **not** land a **major** (47 → 48+) as an ordinary Dependabot or remaining-script cut. Major needs a **short upgrade RFC** (motive, API diff, regression list, rollback pin) per tracking §4.1.
- Do **not** bump `native/Cargo.toml` `wasmtime` on a docs-only eval unless the eval PR **is** the patch upgrade.
- Do **not** edit hub files on a pin/upgrade PR: root `README.md` / `README.zh.md`, `CHANGELOG.md`, `.github/workflows/ci.yml`, `CONTRIBUTING.md`. (Policy PRs that change the living queue may touch README plan tables.)
- Do **not** crate-`cargo fmt`. rustfmt **only** `.rs` files this slice changed.
- Do **not** treat Latch / sync-compat as true CM async (NG-8).
- Never file GitHub issues on Wasmtime, WASI, or any other upstream. No `gh issue create`.

Keep true `async` wraps (`func_wrap_concurrent` + yield) for WIT `async func`. `rustc` **1.97.1**.

## Lanes (auto)

| PR | Sentinel (remaining drops the lane when this leaves the tracking table) | DoD |
|----|------------------------------------------------------------------------|-----|
| **P2-EVAL** | `docs/scheme/wasmtime-tracking.md` still has `gap: p2 pin eval pending` | Refresh §2 / §3: last-checked date, pin (still 47.0.4 unless this PR **is** a patch), upstream latest stable at check. Changelog fragment. Do **not** open a major bump. If a **patch** is justified, land it in the **same** PR (Cargo + lockfile + existing WASI smokes) **or** leave a new `gap: p2 patch pending` needle and stop. Remove `gap: p2 pin eval pending`. |
| **P2-PATCH** | tracking table has `gap: p2 patch pending` | `47.0.4` → `47.0.x` per §4.1: Cargo.toml + lockfile; `cargo check --locked --lib`; existing `wasi_*` native smokes that already run on CI; changelog; update tracking §2 / §3 and `tech-stack.md` if the pin string changes. Remove the needle. |

P2-EVAL landed **2026-08-26**: pin **47.0.4**; crates.io latest stable at check was 48.0.1. Do **not** auto-cut major.

## Named-only (never `Next:`)

| Lane | When | DoD |
|------|------|-----|
| Major upgrade RFC | User named 48+ / major | Short RFC first; dual-ABI before merge; rollback pin in the RFC |
| Enable `wasmtime-wasi` crate | User named wasmtime-wasi | Size + Android thread review in-repo; still one lane |
| P1 leftover WASI shapes | User named G-err / G-cmd / G-fs-full / G-sock-rest / G-http-body / G-http-ctor / G-cli-error | Follow archived P1 playbook + [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md); not auto |
| Full wasi-testsuite | User named testsuite | Optional subset only; no compliance claim |
| Frame-loop / wasi-gfx | User named those pages | Separate RFC; not P2 remaining |

## File whitelist (typical P2-EVAL / P2-PATCH)

- `docs/scheme/wasmtime-tracking.md` / `.zh.md`
- `docs/scheme/tech-stack.md` / `.zh.md` (pin string only, if changed)
- `changelog/unreleased/<date>-*.md`
- `native/Cargo.toml` + `native/Cargo.lock` (**patch PR only**)
- `docs/build.md` only if the documented pin changes

## Tests

| Kind | Command |
|------|---------|
| Docs-only (this close-out, or EVAL with no crate bump) | `python3 ./scripts/wasmtime-p2-remaining.py` |
| Patch crate bump | `cd native && cargo check --locked --lib` + existing CI `wasi_*` filters; do **not** add a new instrument |
| Major | RFC + tracking §4.2 minimum regression (native Android arm64 load + one CM async smoke) |

Cloud has no device. Do not claim a device pass on a pin PR.

## PR title

- Docs eval: `docs: P2 Wasmtime pin eval`
- Patch: `chore(native): wasmtime 47.0.x`
- This close-out: `docs: close P1 WASI 0.3 and start P2 Wasmtime tracking`

Label: `documentation` for docs-only; `dependencies` if Cargo changes (Dependabot already uses that).

## Copy source

Tracking policy is [`../scheme/wasmtime-tracking.md`](../scheme/wasmtime-tracking.md). Do not rediscover it from Wasmtime release notes by rewriting the playbook. Record facts in the tracking table + changelog fragment.
