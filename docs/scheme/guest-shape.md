# Guest shape (`wasi:webgpu`)

**English** | [中文](guest-shape.zh.md)

Canonical guest ABI for this repo. Pin: **`wasi:webgpu@0.3.0-rc.2`** (vendored [`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit)). **P0 is closed** ([`../archive/p0-wasi-webgpu.md`](../archive/p0-wasi-webgpu.md)). Holes vs androidx: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md). Current work: [`../agent/product-010.md`](../agent/product-010.md).

## 1. A method is shape-complete iff

| Axis | Pass | Fail (frozen transitional) |
|------|------|----------------------------|
| Name | `[method]gpu-*.…` matches WIT | Flat `request-adapter`, experimental names, or WIT name with args dropped |
| self | `borrow<resource>` | No self / dummy `get-*` as the only entry while methods still take `u32` |
| Return | `own<resource>` / `option` / `result` / `list` / `string` / void per WIT | Always `u32` or ignored |
| Args | Guest-supplied record / list / option is marshalled | Host-fixed descriptor; guest args discarded |
| async | WIT `async func` → `func_wrap_concurrent` + real yield | Sync wrap / Latch fake-async |

Test-only constructors (`get-gpu`, `get-device`) may remain in fixtures. They **must not** be the product WIT surface. Product slices chain from `gpu.request-adapter`.

## 2. Marshalling

1. Use Wasmtime component types (`Resource<T>`, `Option`, `Result`, lists, records).  
2. Finite, explicit codecs per slice — no schema-free JSON.  
3. Backend GPU handles may stay `u32` **reps**; the **guest boundary** is a resource.  
4. `result` / `option` follow WIT; do not panic on failure and call it a result.  
5. **Forbidden:** new slices whose acceptance is host-fixed descriptor + transitional `u32`.

## 3. S-series (closed)

S1–S5 and S6+ hang + L2 described JNI landed in 2026-08. Do not open new host-fixed `u32` feature PRs. Do not re-cut F1–F9 / G1–G9 / WG-6.

**Hard gate (still):** If S2 cannot be true async, stop expanding `option`/`result` until the pump is fixed.

**No backend / no adapter:** S2 returns **`none`**. See [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md).

## 4. Present / canvas

`wasi:webgpu` has **no** `present`. Product canvas shape is `gpu-canvas-context`. **Continuous on-screen loop for `0.1.0`** is [`rfc-wasi-gfx-frame-loop.md`](rfc-wasi-gfx-frame-loop.md) (not P0; NG-9). Shape notes: [`../mapping/frame-loop-suggestion.md`](../mapping/frame-loop-suggestion.md).
