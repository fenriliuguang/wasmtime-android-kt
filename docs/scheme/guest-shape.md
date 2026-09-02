# Guest shape (`wasi:webgpu`)

**English** | [中文](guest-shape.zh.md)

Canonical guest ABI. Pin: **`wasi:webgpu@0.3.0-rc.2`** ([`../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit`](../../third_party/wasi-webgpu/v0.3.0-rc.2/wit/webgpu.wit)). JNI leftover holes: [`../mapping/gap-webgpu-wit-androidx.md`](../mapping/gap-webgpu-wit-androidx.md). Living consume leftover: [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md). Policy: [`rfc.md`](rfc.md).

## 1. A method is shape-complete iff

| Axis | Pass | Fail |
|------|------|------|
| Name | `[method]gpu-*.…` matches WIT | Flat `request-adapter`, experimental names, or WIT name with args dropped |
| self | `borrow<resource>` | No self / dummy `get-*` as the only entry while methods still take `u32` |
| Return | `own<resource>` / `option` / `result` / `list` / `string` / void per WIT | Always `u32` or ignored |
| Args | Guest-supplied record / list / option is marshalled | Host-fixed descriptor; guest args discarded |
| async | WIT `async func` → `func_wrap_concurrent` + real yield | Sync wrap / Latch fake-async |

Test-only constructors (`get-device`, …) may remain in fixtures. They **must not** be the product WIT surface. Product slices chain from `gpu.request-adapter`.

## 2. Marshalling

1. Use Wasmtime component types (`Resource<T>`, `Option`, `Result`, lists, records).
2. Finite, explicit codecs — no schema-free JSON.
3. Backend GPU handles may stay `u64` Dawn slots or `u32` reps; the **guest boundary** is a resource.
4. `result` / `option` follow WIT; do not panic on failure and call it a result.
5. **Forbidden:** new slices whose acceptance is host-fixed descriptor + transitional `u32`.

**No backend / no adapter:** `request-adapter` returns **`none`**.

## 3. Present / canvas

`wasi:webgpu` has **no** `present`. Product canvas is `gpu-canvas-context`. Continuous on-screen loop is `wasi-gfx:surface@0.2.0` ([`rfc.md`](rfc.md) §3).
