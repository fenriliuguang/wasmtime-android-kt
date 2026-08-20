### Code — L2 gpu-render-pass occlusion and execute-bundles guest fields to host (2026-08-20)

- Deepen `[method]gpu-render-pass-encoder.begin-occlusion-query` / `end-occlusion-query` / `execute-bundles` from lift-only stubs to described JNI (pass handle + query index / bundle reps → Dawn/Cpu)
- Guest `get-pass` / `get-render-bundle` still use rep 0; the wrap stub-builds a clear pass; bundle rep 0 entries are skipped in the attach
- New host APIs `renderPassBeginOcclusionQuery` / `EndOcclusionQuery` / `ExecuteBundles`; Handles gains a RenderBundle kind
