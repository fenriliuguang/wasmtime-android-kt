# RFC: Product, GPU host, and gfx loop

**Status: Accepted** · 2026-09-02 (merged from the former L5 / ecosystem / pluggable-GPU / gfx-loop RFCs)

English is canonical. Short Chinese: [`rfc.zh.md`](rfc.zh.md).

This is the **accepted** product RFC. Guest ABI rules stay in [`guest-shape.md`](guest-shape.md). Draft design RFCs (not policy until accepted): [`rfc-threads.md`](rfc-threads.md), [`rfc-wasi-p3.md`](rfc-wasi-p3.md).

## 1. Product

This repo is an **Android-first app runtime** (class B): upstream Wasmtime, Component Model, true CM async. First proposal world is pinned `wasi:webgpu@0.3.0-rc.2`. Mid-term success is a **citable** host (reproduce + cite + local notes). **Never** file GitHub issues on Wasmtime, WASI, Dawn, androidx, or any other upstream.

| Question | Decision |
|----------|----------|
| Versioning | Perpetual **`0.x.y`** until `wasi:webgpu` and `wasi-gfx` are ratified WASI, WASI publishes **1.0**, and `androidx.webgpu` ships a non-alpha release. Break in MINOR. |
| Coordinate | **`0.1.2`**. **`SNAPSHOT` is allowed** (Central publishing limits). Later bumps follow [`api-stability.md`](api-stability.md). Publishing CI: [`.github/workflows/publish.yml`](../../.github/workflows/publish.yml). Do not press when secrets, arm64 `libwasmtime_android_kt.so`, or arm64 `libwebgpu_dawn.so` are missing. No `0.0.x-preview` Central. |
| Engine | Official `wasmtime` **47.x** only. True CM async only. |
| WASI claim | Product subset, not wasi-testsuite / “full WASI 0.3”. |
| WebGPU claim | Most of the pinned WIT instantiates; Dawn holes listed. **Not** CTS (NG-5). |
| Artifacts | `runtime` (no Dawn) / `host-dawn` / **`android-webgpu` default**. `runtime-api` / `runtime-jni` are Maven transitives only. |

groupId: `io.github.fenriliuguang.wasmtime.android`. Tags `v0.x.y` / `v0.x.y-SNAPSHOT` match the coordinate.

## 2. GPU host

Default consume is **in-process Dawn C** (`NativeGpu`, `GpuBackends.dawn()`, id `"dawn"`). One Dawn `.so` in the default APK. androidx JNI is leftover `id = "dawn-jni"`. Not a second renderer (NG-7). wgpu-native is not the default.

| Question | Decision |
|----------|----------|
| Unwired store | Linker always defines `wasi:webgpu`. `request-adapter` → guest **`none`**. |
| Attach | Explicit `Store.setWebGpuBackend` always wins. `Store.createWithDiscoveredBackend` is default-bundle convenience. |
| SPI | Kotlin `WebGpuBackend` stays BYO / discover. Do not grow it into a 224-method Kotlin WebGPU client (NG-3). |
| Guest shape | Pin `[method]` names, `own`/`borrow`, `async` wraps. No new host-fixed `u32` (NG-12). No JS-style `start(callback)`. |

`libwebgpu_dawn.so` is packed into published `host-dawn` / `android-webgpu` from **0.1.1** (wrong `--build` pin) and **0.1.2** (device-green `--prebuilt`). Cloud CI assemble without the recipe stays table-backed (no ART/JNI). Pin methods that Dawn C can implement call `webgpu.h` when the `.so` is loaded — [`../mapping/gap-webgpu-native-dawn.md`](../mapping/gap-webgpu-native-dawn.md).

## 3. Frame loop (`wasi-gfx`)

Continuous present uses **`wasi-gfx:surface@0.2.0`** (vendored [`../../third_party/wasi-gfx/v0.2.0/wit/surface.wit`](../../third_party/wasi-gfx/v0.2.0/wit/surface.wit)), not a scheduler inside `wasi:webgpu`. Guest **pulls** `on-frame` as a CM `stream`. Host writes vsync on GpuThread (Choreographer 1-slot; unconsumed beats drop; `surfaceDestroyed` closes). `run` stays async. Thread contract: [`../mapping/threading-android.md`](../mapping/threading-android.md).

Landed product loop: constructor + `on-frame` + `height` / `width` / `request-set-size` / `on-resize` + `on-pointer-*` / `on-key-*` + `surface-webgpu` `configure` / `get-current-texture` / `present`. Hitch invariants: keep-3, Fifo, H8 no-op present.

**Remaining (auto):** none.

**Non-urgent (named, never auto):** `context.unconfigure`; timestamped `frame-event`; Lost/Outdated as `result`; multi-window / desktop gfx.

## 4. `0.1.0` subset (not testsuite)

Claim table: [`claim-010.md`](claim-010.md).

Must: most pin `[method]` names; Dawn path for compute / 3D / present when the `.so` is present; documented Record holes; product cli/fs/outbound TCP/HTTP body+send; gfx pull-stream loop; out-of-tree demo link + one named device row.

Not this **0.1.0** gate: full `wasi:cli/command` (G-cmd), G-fs-full, listen/UDP/DNS, wasi-testsuite, `wasmtime-wasi` (needs size + Android thread note), CTS, this-repo 1.0.

After `0.1.2`, named leftovers may be filled on **`cursor/wasi-p3-leftover-b677`** ([`wasi-p3-leftover.md`](wasi-p3-leftover.md)). That queue is still not wasi-testsuite and still not `wasmtime-wasi`. Draft: [`rfc-wasi-p3.md`](rfc-wasi-p3.md).

## 5. Public SPI

Product: `Engine`, `Store`, `Linker`, `Component`, `Instance`; `WasmtimeException`; `WebGpuBackend` / factory / kind; `GpuBackends.dawn()`; native loader / version; thread contract.

Not public SPI: `ExperimentalHostCallbacks`, fixtures, `get-device` / HTTP request-response constructors on `Linker.create`.

## 6. This-repo 1.0.0

Do not start a 1.0 list here. Preconditions: ratified `wasi:webgpu` + `wasi-gfx`, WASI 1.0, stable `androidx.webgpu`.
