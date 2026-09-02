# Scheme index

**English** | [中文](README.zh.md)

Living plan only. Language: [`../LANGUAGE.md`](../LANGUAGE.md).

## Now

Remaining close-out: **Dawn C full bind**, then **wasi-gfx size/resize**, then **remaining pin input streams**. Playbook: [`../agent/remaining.md`](../agent/remaining.md). `python3 ./scripts/remaining.py`. Product coordinate **`0.1.0`** (not released). P2 Wasmtime pin is **named**.

| Doc | Role |
|-----|------|
| [`../agent/remaining.md`](../agent/remaining.md) | Living close-out playbook |
| [`remaining.md`](remaining.md) | Needles (BIND → GFX-SIZE → GFX-PIN) |
| [`rfc.md`](rfc.md) | **Accepted:** product / GPU host / gfx loop |
| [`claim-010.md`](claim-010.md) | `0.1.0` claim table (not CTS) |
| [`guest-shape.md`](guest-shape.md) | WIT shape gates |
| [`charter.md`](charter.md) | Vision / principles |
| [`non-goals.md`](non-goals.md) | Hard no |
| [`api-stability.md`](api-stability.md) | Perpetual `0.x` until upstream 1.0 |
| [`tech-stack.md`](tech-stack.md) | Wasmtime / JNI / NDK |
| [`vcs-workflow.md`](vcs-workflow.md) | Short-lived branches + PR |
| [`wasmtime-tracking.md`](wasmtime-tracking.md) | Engine pin / upgrade |
| [`../agent/wasmtime-p2.md`](../agent/wasmtime-p2.md) | P2 playbook (named) |
| [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md) | WIT ↔ NativeGpu ↔ Dawn C |
| [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md) | JNI leftover holes |
| [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) | WASI 0.3 named leftovers |
| [`../mapping/threading-android.md`](../mapping/threading-android.md) | Thread contract |
| [`../blocked-gpu-host.md`](../blocked-gpu-host.md) | Dawn vendor path |
| [`../build.md`](../build.md) | How to build |
| [`../contribute.md`](../contribute.md) | Contributor shell |

## Principles

1. Canonical `wasi:webgpu` guest shape; no host-fixed `u32` as new-slice acceptance.
2. True CM async via upstream Wasmtime.
3. Android-first; one Dawn as the default backend; do not rewrite Dawn (NG-7).
4. Experimental `0.x`; **no Central before `0.1.0` is pressed**; no CTS claim.
5. English docs are canonical.
