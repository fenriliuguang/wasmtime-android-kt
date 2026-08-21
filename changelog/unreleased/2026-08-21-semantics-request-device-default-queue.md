### Code — leftover descriptor semantics request-device default-queue (2026-08-21)

- JNI `adapterRequestDeviceDescribed` now packs guest `default-queue` label (empty = none) with existing first required-feature / required-limits rep / device label
- Fixture `webgpu_method_request_device` passes `default-queue.label=l2`; native smoke asserts the lifted option (true async wrap kept)
- Dawn `GPUDeviceDescriptor` stays label + callbacks this cut; `defaultQueueLabel` stays on the Kotlin record if androidx omits the ctor slot
