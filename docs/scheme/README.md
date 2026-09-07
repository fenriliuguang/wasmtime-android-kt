# Scheme index

**English** | [中文](README.zh.md)

Product RFC and shape docs. Language: [`../LANGUAGE.md`](../LANGUAGE.md). Coordinate **`0.1.2`**.

| Doc | Role |
|-----|------|
| [`rfc.md`](rfc.md) | **Accepted:** product / GPU host / gfx loop |
| [`rfc-threads.md`](rfc-threads.md) | **Draft:** guest concurrency / stackful CM async (sibling short PR) |
| [`rfc-wasi-p3.md`](rfc-wasi-p3.md) | **Draft:** leftover fill vs completeness |
| [`wasi-p3-leftover.md`](wasi-p3-leftover.md) | Living WASI leftover `L-*` queue (thin host) |
| [`claim-010.md`](claim-010.md) | 0.1.x claim table (not CTS) |
| [`guest-shape.md`](guest-shape.md) | WIT shape gates |
| [`charter.md`](charter.md) | Vision / principles |
| [`non-goals.md`](non-goals.md) | Hard no |
| [`api-stability.md`](api-stability.md) | Perpetual `0.x` until upstream 1.0 |
| [`tech-stack.md`](tech-stack.md) | Wasmtime / JNI / NDK pins |
| [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md) | WIT ↔ NativeGpu ↔ Dawn C |
| [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md) | JNI leftover holes |
| [`../mapping/gap-wasi-p3-wit.md`](../mapping/gap-wasi-p3-wit.md) | WASI 0.3 leftover map (`L-*`) |
| [`../mapping/threading-android.md`](../mapping/threading-android.md) | Thread contract |
| [`../mapping/errors.md`](../mapping/errors.md) | Error / trap contract |
| [`../blocked-gpu-host.md`](../blocked-gpu-host.md) | Dawn vendor path |

## Principles

1. Canonical `wasi:webgpu` guest shape; no host-fixed `u32` as new-slice acceptance.
2. True CM async via upstream Wasmtime.
3. Android-first; one Dawn as the default backend; do not rewrite Dawn (NG-7).
4. Experimental `0.x`; **`0.1.2` current** (`0.1.0` pressed without packing Dawn C; `0.1.1` packed the wrong `--build` Dawn); SNAPSHOT allowed; no CTS claim.
5. English docs are canonical.
