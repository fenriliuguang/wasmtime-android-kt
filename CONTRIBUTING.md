# Contributing

**English** | [中文](CONTRIBUTING.zh.md)

Experimental Android-first Wasm Component runtime. Collaboration is **short-lived branches + pull requests**. Do not push `main` directly.

Language: English is canonical ([`docs/LANGUAGE.md`](docs/LANGUAGE.md)).

## Read first

| Doc | Content |
|-----|---------|
| [`docs/scheme/rfc.md`](docs/scheme/rfc.md) | Product / GPU host / gfx loop |
| [`docs/scheme/wasi-p3-leftover.md`](docs/scheme/wasi-p3-leftover.md) | WASI leftover long branch (after `0.1.2`) |
| [`docs/scheme/guest-shape.md`](docs/scheme/guest-shape.md) | wasi:webgpu WIT gates |
| [`docs/scheme/non-goals.md`](docs/scheme/non-goals.md) | Hard no |
| [`docs/scheme/claim-010.md`](docs/scheme/claim-010.md) | 0.1.x product subset |
| [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md) | GPU host — vendor path |

## Workflow

1. From latest `main`: `docs/…` / `feat/…` / `fix/…` / `chore/…`.  
2. **One PR, one thing.** User-visible changes: new file [`changelog/unreleased/<yyyy-mm-dd>-<slug>.md`](changelog/unreleased/README.md). **Do not** edit root `CHANGELOG.md`.  
3. CI green, then squash-merge; delete the head branch.  
4. No long-lived `feature/*` forks. **`release/0.1.0`** may stay as a maintenance branch; it never uploads Maven. Named exception: **`cursor/wasi-p3-leftover-b677`** (WASI 0.3 leftover fill) — one lane = one commit; open **one** PR to `main` when `python3 ./scripts/wasi-p3-leftover-remaining.py` is empty. Playbook: [`docs/scheme/wasi-p3-leftover.md`](docs/scheme/wasi-p3-leftover.md).

## Hub freeze

These files collide on every short PR. Feature PRs **must not** touch them unless the PR *is* a policy change:

| Hub | Feature PRs | Instead |
|-----|-------------|--------|
| [`CHANGELOG.md`](CHANGELOG.md) | No Unreleased edits | `changelog/unreleased/<date>-<slug>.md` |
| [`.github/workflows/ci.yml`](.github/workflows/ci.yml) | Do not append `--test` | Add `native/tests/<name>.rs` |
| [`.github/workflows/publish.yml`](.github/workflows/publish.yml) | Do not add per-slice publish | Maintainer press only |
| This file | Do not copy test names | Describe job intent only |
| [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md) | Do not churn examples | Keep stable |
| [`README.md`](README.md) / [`README.zh.md`](README.zh.md) | Do not add a row per slice | Topic pages + Project board |

## CI

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) (required check name **`CI`**) on `main` and `release/0.1.0`. GAV may be `0.x.y` or **`0.x.y-SNAPSHOT`**; assemble does not reject SNAPSHOT.

| Job | What |
|-----|------|
| `detect (change set)` | `*.md` + `changelog/**` only → skip the four jobs below |
| `native (cargo test)` | `native/` `cargo test --locked --tests` |
| `jvm (runtime-api compile)` | `:runtime-api` / `:runtime-jni` `compileKotlin` |
| `android (ndk .so)` | `scripts/build-native-android.ps1` (arm64 + x86_64) |
| `android (AAR + smoke APK)` | assemble published AARs + `:smoke-app:assembleDebug` |

Any other path runs the four heavy jobs. Docs-only still reports the required check **`CI`** as success (heavy jobs **skipped**, not missing). GitHub-hosted CI does **not** run device instruments. Publish gate is local:

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
.\scripts\verify-press-aar.ps1 -Assemble
.\scripts\verify-examples-gate.ps1
```

## Publish

- Current GAV is **`0.1.2`**. Later versions follow [`docs/scheme/api-stability.md`](docs/scheme/api-stability.md).
- **`SNAPSHOT` is allowed.** Maven Central publishing limits apply to *releases*; a `-SNAPSHOT` press does not consume that quota and may be overwritten. Use it when a release GAV would hit the limit. A later SNAPSHOT or PATCH is a separate press. Still no `-rc` GAV. No `0.0.x-preview`.
- [`.github/workflows/publish.yml`](.github/workflows/publish.yml) uploads only from **`main`**: annotated tag `v*` (including `v0.x.y-SNAPSHOT`) or `workflow_dispatch`. GitHub Environment **`release`** (required reviewer; allowed refs: `main`, tags `v*`).
- The job cross-compiles wasmtime `.so` at opt-level **2** **and** links Google Android `--prebuilt` `libwebgpu_dawn.so`, then publishes. Missing arm64 wasmtime or Dawn C `.so` **fails** (does not skip). Maven Central secrets missing **fails** if Central is requested. SNAPSHOT goes to the Central Portal **snapshots** repo (`https://central.sonatype.com/repository/maven-snapshots/`); vanniktech routes this from the `-SNAPSHOT` version.
- **Never** run Publish from `release/0.1.0`. Never publish `:smoke-app`.
- Approver checklist: full `:smoke-app:connectedDebugAndroidTest` green on a named device; in-tree **`verify-press-aar.py`** (release AAR `.so` SHA matches recipe). Out-of-tree cube via includeBuild is demo evidence only — it is not the Maven consume path.

New integration tests: only add `native/tests/<name>.rs`. Do not edit `ci.yml`. Do not dispatch `Publish` from a feature PR.

## Conduct

- experimental: no compliance / production claims without an RFC.  
- Never fake WASI 0.3 / CM async with sync-compat.  
- Contributions under **Apache License 2.0**.
