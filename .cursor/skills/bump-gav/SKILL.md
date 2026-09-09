---
name: bump-gav
description: Bumps this repo's Maven GAV (`wasmtime.android.version`) and current-coordinate docs for a SNAPSHOT or non-SNAPSHOT cut. Use when the user asks to bump the version, bump GAV, raise the coordinate, cut `0.x.y` / `0.x.y-SNAPSHOT`, or align consume README with a new press version.
---

# Bump GAV

In-tree coordinate bump only. Do **not** tag, dispatch Publish, roll `CHANGELOG.md`, or bump the `wasmtime` crate.

## Workflow

1. From latest `main`: `chore/<gav>` (example: `chore/0.1.3-snapshot`). One PR, one thing.
2. Set `NEW` from the user. Default: keep press pin (`--prebuilt` `libwebgpu_dawn.so`, wasmtime opt-level 2).
3. Edit **current coordinate** files. Leave **historical** `0.x.y` sentences alone.
4. Add `changelog/unreleased/<yyyy-mm-dd>-chore-bump-<slug>.md`. Do **not** edit root `CHANGELOG.md` or someone else's fragment.
5. Grep for leftover *current* uses of `OLD` (Coordinate / Current GAV / `Use \`OLD\`` / consume `implementation(...)`).
6. Do not commit unless asked. This PR **is** policy: README / CONTRIBUTING current-GAV lines are in scope. Still do **not** touch `CHANGELOG.md`, `ci.yml`, `publish.yml`, or `.github/PULL_REQUEST_TEMPLATE.md`.

## Sources of truth

| File | Field |
|------|--------|
| `gradle.properties` | `wasmtime.android.version=NEW` plus the comment block |
| `native/Cargo.toml` | `version = "NEW"` (same string as Maven, including `-SNAPSHOT`; crate `publish = false`) |
| `native/Cargo.lock` | only package `name = "wasmtime-android-kt"` |

## Always update (current coordinate)

- `CONTRIBUTING.md` / `CONTRIBUTING.zh.md` — "Current GAV"
- `README.md` / `README.zh.md` — status line, `## Use` / `## 使用` heading, `implementation(...)`, BYO `runtime` / `host-dawn` coords, pack sentence for **this** GAV
- `docs/scheme/api-stability.md` (+ `.zh.md`) — "Current coordinate"
- `docs/scheme/charter.md` (+ `.zh.md`) — "coordinate"
- `docs/scheme/README.md` (+ `.zh.md`) — "Coordinate"; history parenthetical may mention older presses
- `docs/scheme/rfc.md` table **Coordinate** row (+ `.zh.md` 产品坐标)
- `docs/scheme/claim-010.md` (+ `.zh.md`) — "Current GAV" / "Maven **NEW** packs"
- `docs/blocked-gpu-host.md` (+ `.zh.md`) — `Maven coordinates (NEW)` only

English is canonical. `.zh.md` is a faithful summary of the current coordinate, not a second source of truth.

## Do not rewrite (historical)

Keep older version numbers when they are facts, not the live coordinate:

- "`0.1.2` press packs …", "from **0.1.2**", "`0.1.1` packed `--build`"
- leftover queue "after `0.1.2`" (`wasi-p3-leftover.md`, gap WASI, CONTRIBUTING read-first table)
- `host-dawn/build.gradle.kts` `Press (0.1.2+)`
- `README.md` "Maven 0.1.2+ already packs this"
- `changelog/unreleased/2026-09-03-chore-bump-012.md` and other past fragments
- NG-6 / tech-stack / THIRD_PARTY "from 0.1.2" origin sentences unless they literally say "current GAV"

Do **not** replace every `OLD` in the tree.

## SNAPSHOT vs release consume

**`NEW` is `0.x.y-SNAPSHOT`:** consumers need the Central Portal snapshots repo (this GAV is not on `mavenCentral()`).

`gradle.properties` comment:

```text
# Product coordinate NEW (Central publishing limits: snapshot
# does not consume a release quota). Later bumps: docs/scheme/api-stability.md.
```

README repositories + coords:

```kotlin
repositories {
    google()
    mavenCentral()
    maven("https://central.sonatype.com/repository/maven-snapshots/")
}
implementation("io.github.fenriliuguang.wasmtime.android:android-webgpu:NEW")
```

**`NEW` is `0.x.y` (non-SNAPSHOT):** `google()` + `mavenCentral()` only. Comment that SNAPSHOT remains allowed for later Central quota.

Chinese README: "Central Portal **快照仓**（此 GAV 不在 `mavenCentral()`）".

## Changelog fragment

```markdown
### Chore — bump GAV to `NEW` (YYYY-MM-DD)

- Coordinate **`NEW`** after `OLD`. <SNAPSHOT quota or mavenCentral()>. Same press pin: `--prebuilt` `libwebgpu_dawn.so` plus wasmtime opt-level 2. <consume repo note>.
```

## Out of scope unless asked

- Annotated tag `vNEW` / GitHub Environment `release` / Publish workflow
- Device instruments / `verify-press-aar.py` / examples gate
- Mixing leftover fills, Dawn recipe changes, or `wasmtime` 47→48 (needs RFC)
