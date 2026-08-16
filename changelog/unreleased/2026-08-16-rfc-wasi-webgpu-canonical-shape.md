### Docs — RFC: canonical wasi:webgpu shape; end parallel tracks (2026-08-16)

- Track A (`wasi-webgpu-jvm-mvp`) is a **simple demo** only; this repo no longer follows its experimental flat ABI
- Guest surface moves toward pinned `wasi:webgpu@0.3.0-rc.2` WIT (resource / async / record / list / option / result)
- Freeze host-fixed transitional u32 `[method]` slices (NG-12); next code slice is **S1** (`gpu-device.queue` → `own<gpu-queue>`)
- Spec: `docs/scheme/rfc-wasi-webgpu-canonical-shape.md`
