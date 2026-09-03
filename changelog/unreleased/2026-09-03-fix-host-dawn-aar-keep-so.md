### Fix — do not rewrite Dawn `.so` when packing `host-dawn` AAR (2026-09-03)

- `:host-dawn` now matches `:android`: `useLegacyPackaging` and `keepDebugSymbols` for `libwebgpu_dawn.so`. AGP `stripReleaseDebugSymbols` / 16 KB ELF align was changing the `--prebuilt` binary, so `verify-press-aar.py` failed (`AAR SHA != recipe`) and the published AAR was not the device-green file.
