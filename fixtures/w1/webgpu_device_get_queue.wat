;; W3: wasi:webgpu/webgpu@0.3.0-rc.2#device-get-queue (transitional flat, sync).
;; Not [method]gpu-device.queue. Guest: adapter → device → queue; return queue u32.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "request-adapter" (func async (result u32)))
    (export "adapter-request-device" (func async (param "adapter" u32) (result u32)))
    (export "device-get-queue" (func (param "device" u32) (result u32)))
  ))
  (alias export $webgpu "request-adapter" (func $request-adapter))
  (alias export $webgpu "adapter-request-device" (func $adapter-request-device))
  (alias export $webgpu "device-get-queue" (func $device-get-queue))

  (core module $m
    (import "" "request-adapter" (func $request-adapter (result i32)))
    (import "" "adapter-request-device" (func $adapter-request-device (param i32) (result i32)))
    (import "" "device-get-queue" (func $device-get-queue (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $adapter i32)
      (local $device i32)
      (local.set $adapter (call $request-adapter))
      (local.set $device (call $adapter-request-device (local.get $adapter)))
      (call $device-get-queue (local.get $device))
    )
  )
  (core func $ra_lower (canon lower (func $request-adapter)))
  (core func $rd_lower (canon lower (func $adapter-request-device)))
  (core func $q_lower (canon lower (func $device-get-queue)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "request-adapter" (func $ra_lower))
      (export "adapter-request-device" (func $rd_lower))
      (export "device-get-queue" (func $q_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
