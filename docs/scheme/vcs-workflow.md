# Version control and collaboration

**English** | [中文](vcs-workflow.zh.md)

Companion: [`charter.md`](charter.md) · [`../contribute.md`](../contribute.md).  
Goal: reviewable, revertible, CI-backed; ready for external PRs.

Drafted 2026-08-11.

## 1. Decisions

| Question | Decision |
|----------|----------|
| Default integration | **`main` + short-lived feature branches + PR** |
| Long-lived parallel lines (`feature/stream`, `feature/webgpu`, …) | **No** |
| What “parallel” means | A **few** short-lived PRs at once (usually ≤2–3), each with its own DoD, merging often |
| Merge unit | One PR, one thing; independently revertible |
| `main` | Always buildable; unfinished work is not hidden on a long fork |
| Avoiding docs/CI collisions | **Hub freeze**: feature PRs do not edit `CHANGELOG.md` / `ci.yml` test lists / root README index; use fragments + `cargo test --tests` |

## 2. Why not long-lived branches then a big merge

Hotspots: `native/` JNI, linker registration, thread pump, public Kotlin API, instruments. Long forks:

- Amplify merge conflicts and wreck reviewable history  
- Leave outsiders unsure which line is the base  
- Weaken bisect / per-PR revert  

Strategic parallelism (webgpu / stream / clocks) maps to **scheduled short PRs**, not permanent branches.

## 3. Branch names and lifetime

| Prefix | Use | Lifetime |
|--------|-----|----------|
| `docs/<topic>` | Docs / planning | Delete on merge; prefer &lt; 1 week |
| `feat/<slice>` | Feature slice | Delete on merge; prefer &lt; 2 weeks |
| `fix/<issue>` | Bug | Delete on merge |
| `chore/<topic>` | Tooling / tracking refresh | Delete on merge |

Forbidden:

- Ownerless standing `feature/*` as a second trunk
- Wasmtime **major** bumps on a feature branch (own PR + [`wasmtime-tracking.md`](wasmtime-tracking.md) RFC)
- Using a branch instead of a feature flag to hide a breaking half-product for a long time (`0.x` may break APIs, but must be reviewable and changelog’d)

## 4. PR rules

1. **One PR, one thing.** Example: stream-read smoke must not mix with a Wasmtime major bump.  
2. **Bring evidence:** say which commands ran; native changes follow the tracking minimum regression set.  
3. **Docs ride along** for public behavior / gap / pin changes. Edit only the **topic page** for that slice — not “next cut” tables, root README index, or §7 of this file.  
4. **CHANGELOG:** user-visible behavior = **new file** [`changelog/unreleased/<yyyy-mm-dd>-<slug>.md`](../../changelog/unreleased/README.md). **Do not** edit root [`CHANGELOG.md`](../../CHANGELOG.md) in a feature PR. Maintainers roll with `.\scripts\roll-changelog.ps1`. Elsewhere, “update CHANGELOG” means a fragment.  
5. **Hub freeze:** [`CONTRIBUTING.md`](../../CONTRIBUTING.md). New `native/tests/*.rs` **must not** require a [`ci.yml`](../../.github/workflows/ci.yml) edit (`cargo test --locked --tests` already).  
6. **Merge:** squash to `main` (linear history). After open source, default squash (or rebase to a few clear commits); avoid empty merge bubbles.  
7. **Delete merged branches.**

## 5. Parallel matrix (schedule, not standing branches)

```text
May open short PRs together:
  A  docs / WIT pin / gap tables / Wasmtime tracking refresh
  C  wasi:webgpu W0–W1 (names, resources, docs; future-only ok first)
  D  wasi:clocks / wasi:random tiny surface (usually no stream)

Must land first, then expand:
  B  WASI 0.3 stream primitives (JNI/Kotlin) — higher merge priority than stream-dependent packages

Hard serial gates:
  webgpu S2 (spec async option) fails ⇒ stop expanding option/result
  stream not ready ⇒ do not open cli stdio / large streaming worlds
  Do not open new W3+ host-fixed u32 feature PRs (see guest-shape)
```

| Slice | Parallel with other short PRs? | Notes |
|-------|--------------------------------|-------|
| Docs / pin / tracking | **Yes** | Merge anytime |
| `feat/…-stream-…` | **Prefer single-line** on `native/` hotspots | Then stdio |
| webgpu S-series | **Prefer single-line** vs other webgpu cuts | Shape: [`guest-shape.md`](guest-shape.md) |
| clocks / random | **Yes** | Small |
| cli stdio / fs streams | **No** (wait for stream) | |

Suggest **at most 2–3** unmerged feature PRs at once.

## 6. `main` protection (open-source checklist)

Before accepting external PRs:

- [ ] No direct push to `main` (GitHub **Ruleset**, not classic branch protection): require PR / linear history / block force push  
- [ ] PR review (solo maintainer: Required approvals = **0** is ok; raise to 1 with a second person)  
- [x] CI: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) (`cargo test --locked --tests` + `:runtime-api:compileKotlin`); Ruleset check name **`CI`**  
- [x] [`CONTRIBUTING.md`](../../CONTRIBUTING.md) → this page + [`../contribute.md`](../contribute.md)  
- [x] Issue / PR templates in [`.github/`](../../.github/)  
- [x] License: [`LICENSE`](../../LICENSE) (Apache-2.0) + [`NOTICE`](../../NOTICE) + [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md)  
- [x] Permission wording in CONTRIBUTING — **must still be set** on GitHub Collaborators  

After merging the PR that lands this checklist: enable **Require status checks → `CI`**, then set Enforcement Active.

## 7. Current practice

| Action | Decision |
|--------|----------|
| Create standing `feature/stream`, `feature/webgpu`, `feature/clocks` | **Do not** |
| Landed slices / next cut | **Do not append lists here.** Live: [`../agent/remaining.md`](../agent/remaining.md); P2 (named): [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md); shipped behavior: [`changelog/unreleased/`](../../changelog/unreleased/) |
| Hot files | Still avoid two uncoordinated PRs editing the same `native/` source (especially linker registration); freeze docs/CI hubs per §4 |

## 8. Revisions

- Small: PR + `changelog/unreleased/` fragment.  
- Changing “no long-lived parallel lines”, hub freeze, or merge policy: update this page.
