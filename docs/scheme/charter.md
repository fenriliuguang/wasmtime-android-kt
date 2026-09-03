# Charter

**English** | [中文](charter.zh.md)

Android-first **Java/Kotlin Component runtime** on **upstream Wasmtime**.

First proposal world: canonical [`wasi:webgpu`](https://github.com/WebAssembly/wasi-webgpu) (WIT shape + true CM async). Ratified WASI 0.3 packages land as guests need them.

## Vision

- Component Model as a first-class citizen (not core wasm only).
- True CM async (`func_wrap_concurrent`, futures, streams).
- Bionic / ART / multi-ABI (`arm64-v8a` primary).
- Kotlin-friendly lifecycle, threads, and errors.
- **Citable** experimental host — [`rfc.md`](rfc.md).

Product class **B**, perpetual **`0.x`**, coordinate **`0.1.1`**. Still **not** a compliant wasi:webgpu product (NG-5).

## Principles

1. **Do not rewrite Dawn** (NG-7). Package and adapt **one** Dawn as the default GPU backend; the core runtime AAR omits Dawn. Vendor form: [`../blocked-gpu-host.md`](../blocked-gpu-host.md).
2. **Android-first.** Desktop is a contributor convenience.
3. **Thin JNI.** Engine / store / linker / instance / future / resource.
4. **Official async.** Never treat Latch/`sync-compat` as true CM async.
5. **Canonical guest shape** — [`guest-shape.md`](guest-shape.md). No new host-fixed `u32` slices.
6. **English docs are canonical** — [`../LANGUAGE.md`](../LANGUAGE.md).

## Now

Product subset: [`claim-010.md`](claim-010.md). Wasmtime major bumps stay **named**.

## Claims

Package coordinates stay **`0.x`** until upstream 1.0 gates in [`rfc.md`](rfc.md) §6. Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml). Do not press when secrets or arm64 wasmtime / Dawn C `.so` are missing. No CTS / WASI-1.0 distro marketing.
