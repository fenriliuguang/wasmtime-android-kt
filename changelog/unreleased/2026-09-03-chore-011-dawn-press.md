### Chore — pack NativeGpu `.so` at press as `0.1.1` (2026-09-03)

- Bump GAV to **`0.1.1`**. Publish builds recipe `libwebgpu_dawn.so` (arm64 + x86_64, same Dawn SHA as `androidx.webgpu`) and packs it into `:host-dawn`. Missing arm64 wasmtime or Dawn C `.so` fails the press. Apps consume Maven; they do not rebuild Dawn or republish `androidx.webgpu`.
- Cloud CI assemble still may omit the recipe (table-backed). README / RFC / GPU-host docs state the split.
