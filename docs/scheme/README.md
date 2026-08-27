# Scheme index

**English** | [中文](README.zh.md)

Living plan only. History: [`../archive/README.md`](../archive/README.md).  
Language: [`../LANGUAGE.md`](../LANGUAGE.md).

## Now

P0 `wasi:webgpu` is **closed**. P1 WASI 0.3 is **closed**. Current queue: [`../agent/product-010.md`](../agent/product-010.md) (`0.1.0` product gates). P2 Wasmtime pin is **named**. **L5 is accepted** ([`rfc-l5-productization.md`](rfc-l5-productization.md)).

| Doc | Role |
|-----|------|
| [`../agent/product-010.md`](../agent/product-010.md) | **`0.1.0` playbook** |
| [`claim-010.md`](claim-010.md) | **`0.1.0` claim table** (not CTS) |
| [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md) | P2 playbook (named) |
| [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md) | **Accepted:** citable host; drop old L4 |
| [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md) | **Accepted:** Dawn default bundle; core without Dawn; `request-adapter` `none` |
| [`rfc-l5-productization.md`](rfc-l5-productization.md) | **Accepted:** class B; perpetual `0.x`; Central at `0.1.0` |
| [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md) | **Accepted intent:** `0.1.0` gfx present loop (not P0) |
| [`long-term-plan.md`](long-term-plan.md) | L0–L5 |
| [`guest-shape.md`](guest-shape.md) | WIT shape gates (P0 closed) |
| [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) | P0 wasi:webgpu (closed) |
| [`wasi-p3-surface.md`](wasi-p3-surface.md) | Stub → archived P1 WASI 0.3 cuts |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | Engine pin / upgrade |
| [`charter.md`](charter.md) | Vision / principles |
| [`non-goals.md`](non-goals.md) | Hard no |
| [`tech-stack.md`](tech-stack.md) | Wasmtime / JNI / NDK |
| [`api-stability.md`](api-stability.md) | Perpetual `0.x` until upstream 1.0 (L5) |
| [`vcs-workflow.md`](vcs-workflow.md) | Short-lived branches + PR |
| [`../blocked-gpu-host.md`](../blocked-gpu-host.md) | Vendor: Host Kotlin in-tree; Dawn via androidx.webgpu |
| [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md) | P0 WIT ↔ androidx holes |
| [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) | P1 leftover WIT shapes (named-only) |
| [`../archive/p1-wasi-p3.md`](../archive/p1-wasi-p3.md) | P1 timeline / leftovers |
| [`../mapping/threading-android.md`](../mapping/threading-android.md) | Thread contract |
| [`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md) | P0 timeline / problems |
| [`../build.md`](../build.md) | How to build |
| [`../contribute.md`](../contribute.md) | Contributor shell |

Slice status: [GitHub Project](https://github.com/users/fenriliuguang/projects/1) and [`changelog/unreleased/`](../../changelog/unreleased/). Do not list every cut here.

## Principles (short)

1. Canonical `wasi:webgpu` guest shape; no host-fixed `u32` as new-slice acceptance.  
2. True CM async via upstream Wasmtime — never Latch/`sync-compat` as “true async”.  
3. Android-first; package/adapt Dawn as the **default backend**, do not rewrite Dawn (NG-7).  
4. Experimental `0.x`; **no Central before `0.1.0` gates**; no CTS claim.  
5. English docs are canonical.
