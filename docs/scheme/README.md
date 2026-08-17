# Scheme index

**English** | [中文](README.zh.md)

Living plan only. History: [`../archive/README.md`](../archive/README.md).  
Language: [`../LANGUAGE.md`](../LANGUAGE.md).

## Now

| Doc | Role |
|-----|------|
| [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md) | **Accepted:** citable host; drop old L4 |
| [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md) | **Accepted:** Dawn default bundle; core without Dawn; `request-adapter` `none` |
| [`long-term-plan.md`](long-term-plan.md) | L0–L5 |
| [`guest-shape.md`](guest-shape.md) | WIT shape gates (S-series) |
| [`roadmap-wasi-webgpu.md`](roadmap-wasi-webgpu.md) | P0 wasi:webgpu |
| [`wasi-p3-surface.md`](wasi-p3-surface.md) | Ratified WASI 0.3 cuts |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | Engine pin / upgrade |
| [`charter.md`](charter.md) | Vision / principles |
| [`non-goals.md`](non-goals.md) | Hard no |
| [`tech-stack.md`](tech-stack.md) | Wasmtime / JNI / NDK |
| [`api-stability.md`](api-stability.md) | `0.x-experimental` SemVer |
| [`vcs-workflow.md`](vcs-workflow.md) | Short-lived branches + PR |
| [`../blocked-gpu-host.md`](../blocked-gpu-host.md) | Unpublished GPU host — **code still depends** |
| [`../mapping/threading-android.md`](../mapping/threading-android.md) | Thread contract |
| [`../build.md`](../build.md) | How to build |
| [`../contribute.md`](../contribute.md) | Contributor shell |

Slice status: [GitHub Project](https://github.com/users/fenriliuguang/projects/1) and [`changelog/unreleased/`](../../changelog/unreleased/). Do not list every cut here.

## Principles (short)

1. Canonical `wasi:webgpu` guest shape; no host-fixed `u32` as new-slice acceptance.  
2. True CM async via upstream Wasmtime — never Latch/`sync-compat` as “true async”.  
3. Android-first; package/adapt Dawn as the **default backend**, do not rewrite Dawn (NG-7).  
4. Experimental; no default Central publish; no compliance claim.  
5. English docs are canonical.
