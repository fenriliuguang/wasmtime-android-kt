### Code — S6+ remaining render-pass and first render-bundle-encoder WIT (2026-08-18)

- Hang remaining render-pass recording plus the first render-bundle-encoder commands that still lacked product `[method]` names: `begin-occlusion-query` / `end-occlusion-query` / `execute-bundles` / `set-immediates` and bundle-encoder `finish` / `set-pipeline` / `set-bind-group` / `draw`
- Guest lifts WIT query index, bundle list, empty immediates, option descriptor=none, pipeline/bind-group borrows, and draw options; export `run` returns harness `1`; L2 unused (lift-only, no new JNI)
- Fixtures `webgpu_method_*`; native modules under `wasi_webgpu_method/`; twin instruments assert harness `1`
