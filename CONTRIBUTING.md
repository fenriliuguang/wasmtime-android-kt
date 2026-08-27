# Contributing

**English** | [中文](CONTRIBUTING.zh.md)

Experimental Android-first Wasm Component runtime. Collaboration is **short-lived branches + pull requests**. Do not push `main` directly.

Language: English is canonical ([`docs/LANGUAGE.md`](docs/LANGUAGE.md)).

## Read first

| Doc | Content |
|-----|---------|
| [`docs/scheme/vcs-workflow.md`](docs/scheme/vcs-workflow.md) | Branch names, PR rules, hub freeze |
| [`docs/contribute.md`](docs/contribute.md) | Local build, optional desktop shell |
| [`docs/scheme/rfc-ecosystem-contribution.md`](docs/scheme/rfc-ecosystem-contribution.md) | Citable host; P0 unchanged |
| [`docs/scheme/rfc-pluggable-gpu-backend.md`](docs/scheme/rfc-pluggable-gpu-backend.md) | Dawn default bundle; SPI |
| [`docs/scheme/rfc-l5-productization.md`](docs/scheme/rfc-l5-productization.md) | Product class B; `0.x`; `0.1.0` coordinates |
| [`docs/scheme/rfc-wasi-gfx-frame-loop.md`](docs/scheme/rfc-wasi-gfx-frame-loop.md) | `0.1.0` gfx present loop (not P0) |
| [`docs/agent/product-010.md`](docs/agent/product-010.md) | `0.1.0` remaining queue (complete gfx loop: GFXB → GFXV; last: DEMO) |
| [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md) | P2 Wasmtime pin (named) |
| [`docs/scheme/guest-shape.md`](docs/scheme/guest-shape.md) | wasi:webgpu WIT gates |
| [`docs/agent/wasmtime-p2.md`](docs/agent/wasmtime-p2.md) | P2 Wasmtime pin (named) |
| [`docs/scheme/non-goals.md`](docs/scheme/non-goals.md) | Hard no |
| [`docs/blocked-gpu-host.md`](docs/blocked-gpu-host.md) | GPU host — **vendor path** (Host Kotlin in-tree) |

## Workflow

1. From latest `main`: `docs/…` / `feat/…` / `fix/…` / `chore/…`.  
2. **One PR, one thing.** User-visible changes: new file [`changelog/unreleased/<yyyy-mm-dd>-<slug>.md`](changelog/unreleased/README.md). **Do not** edit root `CHANGELOG.md`.  
3. CI green, then squash-merge; delete the head branch.  
4. No long-lived `feature/*` forks.

## Hub freeze

These files collide on every short PR. Feature PRs **must not** touch them unless the PR *is* a policy change:

| Hub | Feature PRs | Instead |
|-----|-------------|---------|
| [`CHANGELOG.md`](CHANGELOG.md) | No Unreleased edits | `changelog/unreleased/<date>-<slug>.md` |
| [`.github/workflows/ci.yml`](.github/workflows/ci.yml) | Do not append `--test` | Add `native/tests/<name>.rs` |
| [`.github/workflows/publish.yml`](.github/workflows/publish.yml) | Do not add per-slice publish | Maintainer `workflow_dispatch` only |
| This file | Do not copy test names | Describe job intent only |
| [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md) | Do not churn examples | Keep stable |
| [`README.md`](README.md) / [`README.zh.md`](README.zh.md) | Do not add a row per slice | Topic pages + Project board |
| [`docs/scheme/vcs-workflow.md`](docs/scheme/vcs-workflow.md) checklists | Do not append “done slices” | Roadmap / unreleased |

## CI

| Job | What |
|-----|------|
| `native (cargo test)` | `native/` `cargo test --locked --tests` |
| `jvm (runtime-api compile)` | `:runtime-api` / `:runtime-jni` `compileKotlin` + `publishToMavenLocal` |
| `Publish` (manual) | [`.github/workflows/publish.yml`](.github/workflows/publish.yml) — GitHub Packages + Maven Central; skip if secrets or arm64 `.so` missing |

New integration tests: only add `native/tests/<name>.rs`. Do not edit `ci.yml`. Do not dispatch `Publish` from a feature PR.

## Conduct

- experimental: no compliance / production claims without an RFC.  
- Never fake WASI 0.3 / CM async with sync-compat.  
- Contributions under **Apache License 2.0**.
