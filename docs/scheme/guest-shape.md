# Guest shape (`wasi:webgpu`)

**English** | [中文](guest-shape.zh.md)

Canonical guest ABI for this repo. Pin: **`wasi:webgpu@0.3.0-rc.2`** (vendored [`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit), tag `v0.3.0-rc.2`). Agent playbooks: [`../agent/webgpu-shape-slice.md`](../agent/webgpu-shape-slice.md) (hang names), [`../agent/webgpu-semantic-l2.md`](../agent/webgpu-semantic-l2.md) (guest fields → L2).

Extracted from the 2026-08-16 shape RFC (now in [`../archive/`](../archive/README.md)). Dual-product scheduling in that RFC is **not** current; see [`rfc-ecosystem-contribution.md`](rfc-ecosystem-contribution.md).

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

## 3. S-series order

Live status: GitHub Project. This page only defines order and DoD.

```text
S1  [method]gpu-device.queue : (borrow<gpu-device>) -> own<gpu-queue>
S2  [method]gpu.request-adapter async → option<own<gpu-adapter>>   (hard async gate)
S3  [method]gpu-adapter.request-device async → result<own<gpu-device>, …>
S4  [method]gpu-device.create-buffer with guest gpu-buffer-descriptor
S5  [method]gpu-queue.submit : list<borrow<gpu-command-buffer>>
S6+ Replace remaining frozen transitional methods — shape first, then semantics
    Semantic L2: caller resource, then JNI family (2–4 methods) per PR; see webgpu-semantic-l2.md
    After default L2 remaining is 0: canvas / S1–S3 descriptors / records / cite — see webgpu-midterm.md
```

**Per-slice DoD (S1+):** WIT-isomorphic guest import; `cargo test --locked --tests` if `native/` changes; instrument twin or a written reason; changelog fragment; no new experimental flat names; no compliance claim.

**Hard gate:** If S2 cannot be true async, stop expanding `option`/`result` surface until the pump is fixed.

**No backend / no adapter:** S2 returns **`none`** (WebGPU `requestAdapter()` → `null`). Do not trap, fail instantiate, or invent a guest “resource not found”. Kotlin may still expose `backendKind` for tests. See [`rfc-pluggable-gpu-backend.md`](rfc-pluggable-gpu-backend.md).

## 4. Present / canvas

`wasi:webgpu` has **no** `present`. Product shape is `gpu-canvas-context` (after marshalling). `wasi-gfx` stays a deferred RFC (NG-9). Demo on-screen paths that use experimental surface APIs are **not** product WIT.
