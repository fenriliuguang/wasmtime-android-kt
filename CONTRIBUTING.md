# Contributing

**English** | [中文](CONTRIBUTING.zh.md)

Experimental Android-first Wasm Component runtime. Collaboration is **short-lived branches + pull requests**. Do not push `main` directly.

Language: English is canonical ([`docs/LANGUAGE.md`](docs/LANGUAGE.md)).

## Read first

| Doc | Content |
|-----|---------|
| [`docs/scheme/rfc.md`](docs/scheme/rfc.md) | Product / GPU host / gfx loop |
| [`docs/scheme/guest-shape.md`](docs/scheme/guest-shape.md) | wasi:webgpu WIT gates |
| [`docs/scheme/non-goals.md`](docs/scheme/non-goals.md) | Hard no |
| [`docs/scheme/claim-010.md`](docs/scheme/claim-010.md) | `0.1.0` product subset |
| [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md) | GPU host — vendor path |

## Workflow

1. From latest `main`: `docs/…` / `feat/…` / `fix/…` / `chore/…`.  
2. **One PR, one thing.** User-visible changes: new file [`changelog/unreleased/<yyyy-mm-dd>-<slug>.md`](changelog/unreleased/README.md). **Do not** edit root `CHANGELOG.md`.  
3. CI green, then squash-merge; delete the head branch.  
4. No long-lived `feature/*` forks. **`release/0.1.0`** may stay as a maintenance branch; it never uploads Maven.

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

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) (required check name **`CI`**) on `main` and `release/0.1.0`:

| Job | What |
|-----|------|
| `native (cargo test)` | `native/` `cargo test --locked --tests` |
| `jvm (runtime-api compile)` | `:runtime-api` / `:runtime-jni` `compileKotlin` |
| `android (ndk .so)` | `scripts/build-native-android.ps1` (arm64 + x86_64) |
| `android (AAR + smoke APK)` | assemble published AARs + `:smoke-app:assembleDebug` |

GitHub-hosted CI does **not** run device instruments. Publish gate is local:

```powershell
.\gradlew.bat :smoke-app:connectedDebugAndroidTest
.\scripts\verify-examples-gate.ps1
```

## Publish

- Current GAV is **`0.1.0`** (pressed). Later versions follow [`docs/scheme/api-stability.md`](docs/scheme/api-stability.md). No `SNAPSHOT`, no `-rc` GAV.
- [`.github/workflows/publish.yml`](.github/workflows/publish.yml) uploads only from **`main`**: annotated tag `v*` or `workflow_dispatch`. GitHub Environment **`release`** (required reviewer; allowed refs: `main`, tags `v*`).
- The job cross-compiles arm64 `.so` then publishes. Missing `.so` **fails** (does not skip). Maven Central secrets missing **fails** if Central is requested.
- **Never** run Publish from `release/0.1.0`. Never publish `:smoke-app`.
- Approver checklist: full `:smoke-app:connectedDebugAndroidTest` green on a named device; out-of-tree cube `installDebug` via includeBuild.

New integration tests: only add `native/tests/<name>.rs`. Do not edit `ci.yml`. Do not dispatch `Publish` from a feature PR.

## Conduct

- experimental: no compliance / production claims without an RFC.  
- Never fake WASI 0.3 / CM async with sync-compat.  
- Contributions under **Apache License 2.0**.
