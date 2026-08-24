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

A **non-plan** sketch for a later gfx `on-frame` stream loop (MoonBit guest / Kotlin vsync wiring): [`frame-loop-suggestion.md`](frame-loop-suggestion.md). Not a P1 lane; does not change NG-9.

## 5. Acceptance hints

- Async path: complete happens under the documented thread model; no data-race smoke.  
- On-screen path: no “wrong thread touched Dawn” crashes.
