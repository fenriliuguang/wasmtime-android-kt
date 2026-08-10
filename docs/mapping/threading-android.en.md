# Android threading (Track B)

[中文](threading-android.md) | **English**

Draft 2026-08-10. Extends Track A [`threading.en.md`](../../../wasi-webgpu-jvm-mvp/docs/mapping/threading.en.md) for true CM async.

## Hard rules (draft)

1. Dawn objects + `processEvents` on one **GpuThread**.  
2. Surface/window configure/present/destroy follow the same policy.  
3. One driver for CM `run_concurrent` / event loop per Store — no multi-threaded concurrent run.  
4. Future completion that touches L2/Dawn must run on GpuThread (or only allow complete there).  
5. Correct JNI attach from Rust async callbacks.  
6. No heavy compile/instantiate on the ART main thread.

## Default model (M2–M4)

UI thread posts Surface events → **GpuThread** owns Dawn + optional CM pump; Rust async runtime only schedules and queues Java callbacks.

## vs Track A

A returns synchronously from host imports (latch). B may return a future and complete later — still must not race Dawn/Surface.
