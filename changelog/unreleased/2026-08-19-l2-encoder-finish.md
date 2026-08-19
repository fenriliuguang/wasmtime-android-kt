### Code — L2 gpu-command-encoder finish guest fields to host (2026-08-19)

- Deepen `[method]gpu-command-encoder.finish` from a host-fixed stub to described JNI (`HostArg` string: command-buffer label)
- Guest passes label=`l2`; native wrap uses encoder `rep` when non-zero and forwards the label into `WasiWebGpuHost`; export `run` returns harness `1`
- Fixture `webgpu_method_command_encoder_finish`; native module `command_encoder_finish`
