### Code — leftover descriptor semantics Dawn consume required-limits (2026-08-21)

- Dawn `GPUDeviceDescriptor.requiredLimits` now takes a `GPULimits` built from snapshotted `DeviceDescriptor.requiredLimits` (empty map → none)
- Do not re-cut `record-option-gpu-size64`; guest kebab-case keys map onto androidx slots; stage-only storage keys androidx omits are skipped
- Fixture `webgpu_method_request_device_required_limits` already lifts `max-bind-groups`=4; this cut copies that map into Dawn
