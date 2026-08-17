# Error model (L1)

**English** | [中文](errors.zh.md)

Stable Kotlin exception types + JNI mapping. experimental.

## Kotlin types

| Type | `Kind` | Typical source |
|------|--------|----------------|
| `WasmtimeApiException` | `API` | null/closed handle; missing callback at register |
| `WasmtimeCompileException` | `COMPILE` | `Component.compile` failed |
| `WasmtimeLinkException` | `LINK` | linker define host / `instantiate` failed |
| `WasmtimeTrapException` | `TRAP` | export trap; includes host-callback failure |
| `WasmtimeException` (base) | same | unified `catch`; old ctor defaults `TRAP` |

Package: `runtime-api` → `io.github.fenriliuguang.wasmtime.android.api`.

## JNI mapping

Rust `native/src/error.rs` throws the matching subclass (`(Ljava/lang/String;)V`).

| Operation | Exception |
|-----------|-----------|
| Engine/Store create failure, null handle, illegal setter | `WasmtimeApiException` |
| `Component::new` | `WasmtimeCompileException` |
| `define_host` / `instantiate_async` | `WasmtimeLinkException` |
| `call*` / `run_concurrent` / missing export | `WasmtimeTrapException` |

## experimental host → GPU backend

| Source | Mapping |
|--------|---------|
| Host exception thrown inside a callback | host Err → guest **trap** → `WasmtimeTrapException` |
| experimental host not registered | trap (at call) |
| Backend returns `GpuHandle(0)` | unexpected; instruments assert non-zero |

The leftover **flat experimental** host does **not** lift host failure into a guest-visible `result`. Canonical `wasi:webgpu` `result`/`option` lives on the S-series path ([`../scheme/guest-shape.md`](../scheme/guest-shape.md)). GPU backend artifacts: [`../blocked-gpu-host.md`](../blocked-gpu-host.md).

## Usage

```kotlin
try {
    instance.callUnitToU32(store, "run")
} catch (e: WasmtimeTrapException) {
    // guest / host-callback trap
} catch (e: WasmtimeException) {
    // compile / link / api
}
```

Kotlin `require` (closed handle) still throws `IllegalArgumentException` — **not** via JNI.

## Out of scope on this page

- Translating traps into guest `result<_,_>` for the experimental flat host  
- Splitting Dawn / host exception subclasses into L1 kinds  
- A stable 1.0 error-code table (still experimental)
