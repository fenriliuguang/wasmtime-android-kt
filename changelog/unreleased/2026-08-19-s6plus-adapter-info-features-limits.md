### Code — S6+ adapter features / limits / info WIT (2026-08-19)

- Hang product `[method]` names: `gpu-adapter.features` / `limits` / `info` and `gpu-adapter-info.vendor` / `architecture` / `device` / `description` / `subgroup-min-size` / `subgroup-max-size` / `is-fallback-adapter`
- Guest lifts WIT own resources (drop) or string/u32/bool getters; export `run` returns harness `1`; L2 unused (empty strings, subgroup sizes 1, fallback false; no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
