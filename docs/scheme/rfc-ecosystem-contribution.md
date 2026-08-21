# RFC: Ecosystem contribution criteria (demote L4)

**Status: Accepted** · 2026-08-17 · **Amended 2026-08-21:** never file GitHub issues on upstream repositories.
**English** | [中文](rfc-ecosystem-contribution.zh.md)

> Amends [`long-term-plan.md`](long-term-plan.md). Does **not** change P0 (`wasi:webgpu` canonical WIT + true Component Model async).  
> WIT shape rules stay in [`guest-shape.md`](guest-shape.md) (extracted from the 2026-08-16 shape RFC, now archived).

## 1. Decision

| Question | Decision |
|----------|----------|
| What is mid-term success? | A **citable Android host**: outsiders can reproduce Guest → this runtime → GPU (or a documented stand-in). Android facts stay in this repo. **Do not** file GitHub issues against [`wasi-webgpu`](https://github.com/WebAssembly/wasi-webgpu), Wasmtime, or any other upstream. |
| Does P0 change? | **No.** Canonical `wasi:webgpu@0.3.0-rc.2` shape + true CM async remain first. |
| Old L4 (“swap another demo’s default runtime”)? | **Dropped** from the goal stack. Not a success criterion. |
| Maven Central / “production runtime”? | Still **not** a near-term goal (NG-5 / NG-6). Citability ≠ Central. |
| `wasi-gfx` / multi-window? | Still **not** P0 (NG-9). |

## 2. Why

Slice count and internal gates do not make this repo a link in the Wasm component chain. Outsiders need:

1. **Reproducible** — `docs/build.md` on a clean machine yields the native library and a passing smoke (JVM and/or device).  
2. **Citable** — one English README sentence, pinned WIT tag, threading contract, no dual-product story on the front door.  
3. **Local notes** — each completed shape cluster may record Android constraints in `changelog/unreleased/` and topic mapping docs (threads, JNI, Bionic). **Never** GitHub Issues or Discussions-as-issues on upstream.

Shipping Maven or a second GPU stack is not required for (1)–(3).

## 3. Goal stack (amended)

```text
L0  Base freeze + Wasmtime tracking
L1  WASI 0.3 primitives (async func, future, stream, concurrent pump)
L2  WASI 0.3 core imports subset (as guests block)
L3  wasi:webgpu canonical WIT (P0) — S-series shape
L4  Citable host (this RFC) — reproduce + cite + local notes
L5  Productization RFC (API freeze candidate, publish-or-not) — still separate
```

**Hard order:** P0 remains L3. L4 may proceed in parallel with late S-series **documentation** (build/embed notes) but must not weaken WIT shape gates. L5 is not implied by L4.

Removed: “optional swap of an external demo runtime” as L4.

## 4. Mid-term success (replace previous wording)

A third party can:

- Explain this repo as **Android + upstream Wasmtime + canonical `wasi:webgpu`**, without a second project’s ABI.  
- Follow [`../build.md`](../build.md) and [`../contribute.md`](../contribute.md) to build.  
- Point at [`guest-shape.md`](guest-shape.md) and [`../mapping/threading-android.md`](../mapping/threading-android.md).  
- See at least one **local** mapping or changelog note from Android host experience — **not** an upstream GitHub issue.

## 5. No upstream GitHub issues

When an S-series cluster lands (resource/`option`/`result`/`list`/`async`):

1. Record the WIT names and Android constraint in `changelog/unreleased/` (and mapping docs if the pump changed).  
2. **Do not** create, reopen, or request GitHub Issues (or GitHub Discussions used as an issue tracker) on wasi-webgpu, Wasmtime, or any other upstream. No `gh issue create`.  
3. Do **not** wait for a full WebGPU CTS pass (NG-5).

## 6. Non-goals this RFC does not lift

- Compliant wasi:webgpu product claim  
- Default Maven Central publish  
- **Rewriting** Dawn (packaging/adapting one Dawn as `:host-dawn` is in [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md))  
- Promoting `wasi-gfx` to P0  
- Host-fixed transitional `u32` as acceptance for new slices  

## 7. GPU host (vendored)

Instrument tests compile against Host Kotlin **inside `:host-dawn`**. Dawn `.so` is published `androidx.webgpu`, not git. Form: [`../blocked-gpu-host.md`](../blocked-gpu-host.md).
