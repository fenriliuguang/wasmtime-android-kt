# Dawn guest compute (WG-6)

- Guest `webgpu_method_dawn_guest_compute` 走 BGL + bind-group + compute pipeline + `set-bind-group` + `dispatch-workgroups` + `queue.submit`，打到 `DawnWasiWebGpuHost`；不是 empty `begin-compute-pass`。
- `ExperimentalWebGpuBridge.attachDawnGuestCompute` 组合转发 described 回调；instrument `WasiWebGpuDawnGuestComputeInstrumentedTest` 资产跑夹具。
- Native `dawn_guest_compute`：shader `read_write`、BGL storage、pipeline specific layout、bind-group buffer、dispatch 1×1×1、submit。
