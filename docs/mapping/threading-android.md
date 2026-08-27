# Android threading

**English** | [中文](threading-android.zh.md)

Draft 2026-08-10; M4 pins 2026-08-11. True Component Model async on ART + a GPU backend.

GPU objects today come from in-tree `:host-dawn` plus `androidx.webgpu` ([`../blocked-gpu-host.md`](../blocked-gpu-host.md)). Dawn is the **default** backend for the product/test bundle ([`../scheme/rfc-pluggable-gpu-backend.md`](../scheme/rfc-pluggable-gpu-backend.md)). The thread rules below still apply to whatever backend owns Dawn / `ANativeWindow`.

## 1. Why a dedicated page

With true CM async:

- A guest may suspend; the host completes a future **later**  
- Dawn / `ANativeWindow` / androidx.webgpu thread rules still apply  
- `run_concurrent` / the event loop and “who owns the Surface” must be explicit

Do not block inside a host import on the export-calling thread waiting for GPU work.

## 2. Hard rules (draft)

1. **Dawn GPU objects and `processEvents`** live on one dedicated **GpuThread**.  
2. **Surface / window handles:** create, configure, present, destroy follow the same GpuThread policy. Do not `windowFromSurface` from arbitrary threads.  
3. **CM concurrent event loop:** one documented driver per Store (`GpuThread` or a named `RuntimeThread`). Never `run_concurrent` the same Store from two threads.  
4. **Future complete:** if the completion closure touches L2/Dawn, **post to GpuThread** first — or allow complete only on GpuThread.  
5. **JNI attach:** Rust async callbacks into Java must `AttachCurrentThread` / detach correctly; no leaked attaches.  
6. **Forbidden:** heavy compile/instantiate on the ART main thread (short `loadLibrary` is ok; long work goes to a background thread and is documented).

## 3. Default model

```text
UI / main thread   : Surface lifecycle only → post to GpuThread
GpuThread          : Dawn + processEvents + present + (optional) CM pump
Rust async runtime : Wasmtime scheduling only; Java callbacks via a thread-safe queue
```

If a later slice proves “CM loop must share GpuThread”, merge them and revise this page.

### Device on-screen smoke (2026-08-11)

- Instrument `DawnRenderSmokeInstrumentedTest`: CM instantiate / host callbacks / Dawn present **all on one GpuThread** (anonymous background thread).  
- Instrument `WasiWebGpuMethodCanvasContextPresentInstrumentedTest`: bind the host-owned window on GpuThread, then guest `gpu-canvas-context.configure` / `get-current-texture` (`run_concurrent`). Host acquires; present of a guest-drawn frame is `WasiWebGpuDawnGuestCanvasPresentInstrumentedTest`.  
- UI thread: Surface lifecycle and Activity start/stop only; `windowFromSurface` on GpuThread.  
- Before starting `MainActivity`, unblank / unlock and use privileged `am start -W` (`targetContext.startActivity` on Android 16 / some OEMs is a background start and never reaches `RESUMED`).  
- First on-screen slice may still be sync-compat (no second CM async pump); keep it isolated from the M2 `run_concurrent` path.

## 4. Async vs blocking host imports

| Point | Blocking host import | Target (true async) |
|-------|----------------------|---------------------|
| Return from host import | Result now | May return a future, complete later |
| Waiting for GPU | Latch + `processEvents` in the callback | Non-blocking request → GpuThread finishes → complete future |
| Acceptance | Historical on-screen smoke | Prove the model first, then frame loops |

A **shape-notes** page for the accepted gfx `on-frame` stream loop (MoonBit guest / Kotlin vsync wiring): [`frame-loop-suggestion.md`](frame-loop-suggestion.md). **`0.1.0` gate** is [`../scheme/rfc-wasi-gfx-frame-loop.md`](../scheme/rfc-wasi-gfx-frame-loop.md). Not a P2 lane; NG-9 still forbids a P0 re-queue.

## 5. WASI 0.3 filesystem sandbox (W6)

`wasi:filesystem` host IO in this repo is **synchronous** inside the import (same class as `system-clock.now`): `std::fs::read` / `write` on a tiny smoke. Do not put large FS work on the ART main thread; the instrument runs on the instrumentation thread.

Path policy:

- Sandbox root: `std::env::temp_dir().join("wasmtime-android-kt-wasi-fs")`.
- Android device: the instrument sets `TMPDIR` to `targetContext.cacheDir` (app-private) before instantiate. Not `/sdcard`, not other world-writable shared storage.
- Guest-relative join (for this cut’s preopen file name and any later `open-at`): reject empty, NUL, `..`, `.`, and absolute/prefix paths (`access`).
- This cut preopens the sandbox **directory** via official `get-directories` → `list<tuple<descriptor, string>>` (length 1, name `"."`). Guest `open-at("p3fs.txt")` then r/w the child. Guest `open-at("..")` is `error-code.access`.

## 6. WASI 0.3 sockets (W7)

`wasi:sockets` this cut is **outbound TCP**: guest `connect(ip-socket-address)` to a non-loopback IPv4, host **dials that address**. Loopback (`127.0.0.1`) still uses the W7 echo pair (port ignored). **No listen / UDP** by default (sandbox). Android needs the **INTERNET** permission for any `TcpStream` (`smoke-app` manifest). Blocking connect / read / write run on a **helper thread**; the CM import is `func_wrap_concurrent` + oneshot (same class as `monotonic-clock.wait-for`). Do not bind product sockets or sleep on the ART main thread.

## 7. WASI 0.3 http (W8)

`wasi:http` this cut is an **in-process** `incoming-handler` ABI smoke (guest `handle` → status 200) plus **body `stream<u8>`** (`consume-body` / `response.new`) and **outbound** `wasi:http/client@0.3.0#send` (HTTP/1.1 GET on the wire). `send` runs on a **helper thread** (same class as TCP connect). Android needs **INTERNET**; smoke-app allows cleartext for the local instrument. No TLS crate this lane (https → `unknown`). Do not add `wasmtime-wasi`.

## 8. wasi-gfx `on-frame` / present (P010-GFXH / P010-GFXL / P010-GFXV)

Product `wasi-gfx:surface@0.2.0` `[method]surface.on-frame` returns a CM `stream<frame-event>`. Guest **pulls**. UI / Choreographer posts vsync into a **1-slot** gate (`Store.postGfxVsync`); if the previous event is unconsumed the beat is **dropped**. `poll_produce` waits on that gate (condvar) and writes the event on the CM driver thread (**GpuThread**). Pin `on-frame` is a sync `func`, not `async func`; this repo does not enable Wasmtime stackful CM async (guest WAT traps on stream.read BLOCKED, so the producer must not return `Poll::Pending`). Do not JS-style `start(callback)`. `surfaceDestroyed` calls `Store.closeGfxOnFrame` so guest `run` unblocks. Product `surface-webgpu` `context.present` presents the pending swapchain texture (idempotent with WG-6 auto-present on `queue.submit`). Multi-frame instrument: `WasiGfxFrameLoopInstrumentedTest`. Do not add `wasmtime-wasi`.

## 9. Acceptance hints

- Async path: complete happens under the documented thread model; no data-race smoke.  
- On-screen path: no “wrong thread touched Dawn” crashes.
