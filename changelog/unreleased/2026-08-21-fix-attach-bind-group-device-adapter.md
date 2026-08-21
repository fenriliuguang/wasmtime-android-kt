### Fix — attach create-bind-group buffer ctor and device.adapter-info owning adapter (2026-08-21)

- `attachCreateBindGroup` wires stub `deviceCreateBuffer` so guest `get-buffer` (rep 0) can fill a bind-group entry
- `attachDeviceInfoError` wires `deviceAdapterDescribed` so `gpu-device.features` / `adapter-info` can store the owning adapter
