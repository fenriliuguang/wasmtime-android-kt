### Code — leftover descriptor semantics request-device required-features list (2026-08-21)

- JNI `adapterRequestDeviceDescribed` now packs the full guest `required-features` list as `[I` WIT ordinals (empty = none); not a new method
- Fixture `webgpu_method_request_device` passes two features (`core-features-and-limits`, `depth-clip-control`) plus existing default-queue label; native smoke asserts the lifted list
- Dawn `GPUDeviceDescriptor.requiredFeatures` takes androidx `FeatureName` ints (`WIT + 1`); empty list stays `intArrayOf()`
