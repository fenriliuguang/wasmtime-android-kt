# Long-term plan

**English** | [中文](long-term-plan.zh.md)

> Current. Amended 2026-08-17 by [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md) and [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md).  
> WIT rules: [`guest-shape.md`](guest-shape.md). History: [`../archive/README.md`](../archive/README.md).

## 1. One sentence

Build an **Android-first JVM Component runtime** that:

1. Hosts **ratified WASI 0.3** capabilities as needed;  
2. Treats **canonical `wasi:webgpu` WIT** as the first proposal world (P0);  
3. Tracks **upstream Wasmtime** only.

Remain experimental. No default publish. No compliance claim.

## 2. Priority (unchanged)

```text
P0  wasi:webgpu canonical shape + true CM async
P1  WASI 0.3 ratified primitives / package subset
P2  Upstream Wasmtime pin + upgrade RFC
```

Proposal work (Phase 2 `wasi:webgpu`) **may** lead implementation and feedback. Wording must stay “proposal host”, not “standard product”.

## 3. Stack L0–L5

```text
L0  Base freeze + Wasmtime tracking
L1  WASI 0.3 primitives (async func, future, stream, concurrent pump)
L2  WASI 0.3 core imports subset (clocks / random / cli / … as guests block)
L3  wasi:webgpu canonical WIT (P0) — S-series
L4  Citable host — reproduce, cite, file upstream notes
L5  Productization RFC (API freeze / publish-or-not)
```

**Dropped:** swapping an external demo’s default runtime as a numbered goal.

**Hard order:** L3 (P0) is not replaced by L4. L4 is documentation and citability, not a weaker WIT gate. L5 is separate.

### Success

| Stage | Looks like |
|-------|------------|
| Near | Docs IA: English front door; archive history; Dawn default bundle accepted; unpublished host still blocked in code |
| Mid | JNI/Kotlin can carry WASI 0.3 `stream` + a small package subset; guests see pinned WIT types (not transitional `u32`); true async on device; **third party can reproduce and cite**; default test APK includes Dawn; at least one upstream-shaped note |
| Far | Outsiders describe this repo as Android + Wasmtime + canonical wasi:webgpu without a second ABI story |

## 4. Principles

1. Do not **rewrite** Dawn (NG-7). Package/adapt Dawn as the default backend; core AAR omits it ([`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)).  
2. Official CM async / WASI 0.3 async — no fake async.  
3. Implementation ≠ compliance claim.  
4. P3 ratified cuts by [`wasi-p3-surface.md`](wasi-p3-surface.md); only wasi:webgpu is P0 among proposals.  
5. Android-first.  
6. New wasi:webgpu slices must be WIT-isomorphic ([`guest-shape.md`](guest-shape.md)).  
7. English is canonical.

## 5. Doc map

| Doc | Role |
|-----|------|
| This page | Strategy |
| [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md) | Citability / L4 |
| [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md) | Dawn default bundle; SPI; `request-adapter` `none` |
| [`guest-shape.md`](guest-shape.md) | Shape + S-series |
| [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) | P0 roadmap |
| [`wasi-p3-surface.md`](wasi-p3-surface.md) | P3 cuts |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | Engine |
| [`../blocked-gpu-host.md`](../blocked-gpu-host.md) | Current unpublished Dawn coordinates (code) |

## 6. Revisions

- 2026-08-16: Canonical WIT (archived RFC).  
- 2026-08-17: Ecosystem citability; remove dual-product L4; English front door; pluggable GPU / Dawn default bundle.
