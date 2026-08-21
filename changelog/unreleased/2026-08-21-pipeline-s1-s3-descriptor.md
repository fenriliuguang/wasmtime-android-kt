### Code — L2 S1–S3 leftover descriptor fields to host (2026-08-21)

- Pass `gpu.request-adapter` `feature-level` (empty = none) through described JNI; keep true CM async
- Pass `gpu-adapter.request-device` `required-limits` `record-option-gpu-size64` **rep** (0 = none) + `label`; snapshot the existing size64 map
- Fixtures `request_adapter_feature_level` (`core`) and `request_device_required_limits` (`max-bind-groups`=4, label=`l2`)
