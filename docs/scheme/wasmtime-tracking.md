# Upstream Wasmtime tracking

**English** | [中文](wasmtime-tracking.zh.md)

Companion: [`long-term-plan.md`](long-term-plan.md) **P2** · [`tech-stack.md`](tech-stack.md).  
Policy: depend only on official `wasmtime` (and explicitly chosen official sibling crates). **No wasmtime4j.**

## 1. What we track

| Category | Content |
|----------|---------|
| Version | crates.io / GitHub `wasmtime` semver |
| Features | `component-model`, CM async, WASI features (list `wasmtime-wasi` separately if enabled) |
| CM async API | `func_wrap_concurrent`, `FutureReader`/`FutureProducer`, `run_concurrent`, **stream** APIs |
| WASI 0.3 | Upstream P3 worlds / `wasmtime-wasi` defaults and breakages |
| Android / link | NDK, page size, binary size, libc; platform notes in release notes |
| Security | RustSec / upstream advisories |

KPI is **knowable, upgradable, rollback-able** — not “always on latest major”.

## 2. Current pin (baseline)

| Item | Value (2026-08-11) | Source |
|------|--------------------|--------|
| `wasmtime` | **47.0.2** | `native/Cargo.toml` |
| Intent | Stay on a current **47.x** generation that supports CM async + WASI 0.3 | [`tech-stack.md`](tech-stack.md) |
| Features (summary) | `component-model` + async (see Cargo.toml) | lockfile at build time |
| Artifacts | `libwasmtime_android_kt.so` / desktop cdylib | [`../mapping/artifacts.md`](../mapping/artifacts.md) |

After an upgrade, update this table, `tech-stack.md`, a changelog fragment, and `docs/build.md` if needed.

## 3. Living tracker

Refresh the “last checked” row when evaluating upstream; do not churn code for every upstream patch.

| Field | Current |
|-------|---------|
| Last checked | 2026-08-11 |
| Pin | 47.0.2 |
| Upstream latest stable (at check) | (fill from crates.io / GitHub; docs-only PRs need not fetch) |
| WASI 0.3 / CM async default | Upstream treats WASI 0.3.0 + CM async as mainline from 46; this repo 47.x already uses CM async |
| Gaps vs long-term plan | **stream** JNI/Kotlin surface not fully exposed; WASI packages not wired through `wasmtime-wasi` |
| Known risks | major may change concurrent APIs; Android cross-compile must regress load + async smoke |
| Next eval trigger | §5 |

### 3.1 Signals

- Dependabot: only direct `wasmtime` in `native/` ([`.github/dependabot.yml`](../../.github/dependabot.yml)); **ignore major** (needs §4.1 RFC)  
- [Wasmtime releases](https://github.com/bytecodealliance/wasmtime/releases)  
- [Bytecode Alliance / WASI 0.3](https://bytecodealliance.org/articles/WASI-0.3)  
- docs.rs: `wasmtime::component` concurrent / stream  
- (Optional) `wasmtime-wasi` P3 features and Android viability  

## 4. Upgrade policy

### 4.1 Levels

| Change | Process |
|--------|---------|
| **patch** (47.0.2 → 47.0.x) | Update Cargo; native build + existing instruments / JVM smoke; changelog fragment; this table |
| **minor** (if upstream ships) | Same as patch + skim component/WASI release notes |
| **major** (47 → 48+) | **Upgrade RFC** (short): motive, API diff, regression list, rollback pin; dual-ABI build before merge |

### 4.2 Minimum regression (upgrade gate)

1. `scripts/build-native-android.ps1` (at least arm64; dual ABI for a release)  
2. Load: `loadLibrary` + `nativeWasmtimeVersion`  
3. True CM async future smoke  
4. If WASI/webgpu slices are wired: at least one matching instrument  
5. `scripts/verify-native-android.ps1` (release or `-RequireAll`)

### 4.3 Forbidden

- Introduce `ai.tegmentum:wasmtime4j` or 4j native as the runtime  
- Major jump without RFC  
- Break the Android main-path build to pick up an experimental WASI feature  

## 5. Cadence

| Trigger | Action |
|---------|--------|
| Dependabot `wasmtime` patch/minor PR | Evaluate per §4.1; update §2 / lockfile / fragment; **do not** land major as ordinary Dependabot |
| Before opening L1 stream / WASI package slices | Check upstream API stability; update §3 |
| Upstream security advisory on `wasmtime` | Evaluate a patch immediately |
| Proposal WIT requires a newer generation | Open a major/minor upgrade RFC |
| Quarterly (suggested) | Refresh “upstream latest stable” even if not upgrading |

## 6. Sibling crates

| Crate | Policy |
|-------|--------|
| `wasmtime` | **Required** |
| `wasmtime-wasi` / WASI preset host | **Optional**; slice RFC first (size, threads, Android FS semantics) |
| `cranelift-*` | Transitive via `wasmtime`; do not pin directly unless debugging |
| Third-party `wasi-webgpu-wasmtime` | **Reference implementation only**; not a cdylib dependency |

## 7. Revisions

- Baseline version: this §2 + tech-stack + Cargo + changelog fragment in the same PR.  
- Changing “official Wasmtime only”: charter-level RFC.
