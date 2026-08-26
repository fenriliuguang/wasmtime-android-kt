# Long-term plan

**English** | [中文](long-term-plan.zh.md)

> Current. Amended 2026-08-17 by [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md) and [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md).  
> Amended 2026-08-26 by [`rfc-l5-productization.md`](rfc-l5-productization.md) (perpetual `0.x`; Central at `0.1.0` gates) and [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md).  
> WIT rules: [`guest-shape.md`](guest-shape.md). History: [`../archive/README.md`](../archive/README.md).

## 1. One sentence

Build an **Android-first JVM Component runtime** that:

1. Hosts **ratified WASI 0.3** capabilities as a **product subset** (**P1 Smoke closed** 2026-08-26; `0.1.0` gate in L5);  
2. Treats **canonical `wasi:webgpu` WIT** as the first proposal world (**P0 closed** 2026-08-22);  
3. Tracks **upstream Wasmtime** only (**P2 current**).

**Product class B** (app runtime). Perpetual **`0.x.y`** until upstream WASI/webgpu/gfx 1.0 and stable `androidx.webgpu` ([`rfc-l5-productization.md`](rfc-l5-productization.md)). No Central before `0.1.0` gates. No CTS claim.

## 2. Priority

```text
P0  wasi:webgpu canonical shape + true CM async     CLOSED
P1  WASI 0.3 ratified primitives / packages + device  CLOSED
P2  Upstream Wasmtime pin + upgrade RFC               current
```

P0 close-out: [`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md). P1 close-out: [`../archive/p1-wasi-p3.md`](../archive/p1-wasi-p3.md). P2 playbook: [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md). L5: [`rfc-l5-productization.md`](rfc-l5-productization.md). Wording: proposal host for webgpu until `0.1.0` claims “most of the pin,” still **not** CTS.

## 3. Stack L0–L5

```text
L0  Base freeze + Wasmtime tracking
L1  WASI 0.3 primitives (async func, future, stream, concurrent pump)
L2  WASI 0.3 core imports subset (clocks / random / cli / … as guests block)
L3  wasi:webgpu canonical WIT (P0) — S-series
L4  Citable host — reproduce, cite, local notes (never upstream GitHub issues)
L5  Productization — perpetual `0.x`; Central at `0.1.0` gates ([`rfc-l5-productization.md`](rfc-l5-productization.md))
```

**Dropped:** swapping an external demo’s default runtime as a numbered goal.

**Hard order:** L3 (P0) is not replaced by L4. L4 is documentation and citability, not a weaker WIT gate. L5 policy is **accepted**; implementation is follow-up, **not** `wasmtime-p2-remaining` `Next:`.

### Success

| Stage | Looks like |
|-------|------------|
| Near | **Done:** English front door; Dawn default bundle; Host Kotlin in `:host-dawn`; P0 webgpu shape |
| Mid | **Done (P1):** official WASI 0.3 package WIT (Smoke subset) on device. **Current (P2):** Wasmtime pin known / upgradeable / rollback-able; **third party can reproduce and cite**; default test APK includes Dawn |
| Far | `0.1.0` app runtime: most pinned wasi:webgpu + product WASI subset + gfx frame loop; still `0.x` until upstream 1.0 |

## 4. Principles

1. Do not **rewrite** Dawn (NG-7). Package/adapt Dawn as the default backend; core AAR omits it ([`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md)).  
2. Official CM async / WASI 0.3 async — no fake async.  
3. Implementation ≠ compliance claim.  
4. P3 ratified cuts archived at [`../archive/p1-wasi-p3-surface.md`](../archive/p1-wasi-p3-surface.md); only wasi:webgpu is P0 among proposals.  
5. Android-first.  
6. New wasi:webgpu slices must be WIT-isomorphic ([`guest-shape.md`](guest-shape.md)).  
7. English is canonical.

## 5. Doc map

| Doc | Role |
|-----|------|
| This page | Strategy |
| [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md) | Citability / L4 |
| [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md) | Dawn default bundle; SPI; `request-adapter` `none` |
| [`guest-shape.md`](guest-shape.md) | Shape gates (P0 closed) |
| [`rfc-l5-productization.md`](rfc-l5-productization.md) | L5: 0.x product class B; `0.1.0` gates; no Central before |
| [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md) | `0.1.0` gfx present loop (not P0) |
| [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) | P0 (closed) |
| [`../archive/p1-wasi-p3-surface.md`](../archive/p1-wasi-p3-surface.md) | P1 WASI 0.3 cuts (archived) |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | Engine |
| [`../blocked-gpu-host.md`](../blocked-gpu-host.md) | Vendor path: Host Kotlin in-tree; Dawn via androidx.webgpu |

## 6. Revisions

- 2026-08-16: Canonical WIT (archived RFC).  
- 2026-08-17: Ecosystem citability; remove dual-product L4; English front door; pluggable GPU / Dawn default bundle.  
- 2026-08-22: P0 closed; P1 WASI 0.3 is the living queue.  
- 2026-08-26: P1 closed; P2 Wasmtime pin is the living engineering queue. Named P1 leftovers stay in [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md).  
- 2026-08-26: L5 accepted — product class B, perpetual `0.x`, Central only at `0.1.0` gates; gfx frame loop is a separate RFC.
