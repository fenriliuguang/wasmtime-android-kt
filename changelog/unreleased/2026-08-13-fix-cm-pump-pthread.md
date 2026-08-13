### Fix — callRunConcurrent 8MiB pthread pump (W3 device-get-queue) (2026-08-13)

- `nativeCallRunConcurrent` pumps Wasmtime on an 8MiB pthread (`wasmtime-cm-pump`) and bounces L2 JNI to the caller thread; Java `Thread` stackSize from #23 is ignored and attaching that pthread aborts ART (`FindStackTop` / Vivo)
