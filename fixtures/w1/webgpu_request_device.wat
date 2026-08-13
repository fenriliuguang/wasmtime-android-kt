;; W2 remainder: wasi:webgpu/webgpu@0.3.0-rc.2#adapter-request-device
;; Transitional flat name (not [method]gpu-adapter.request-device); true CM async.
;; Guest: request-adapter → adapter-request-device → return device u32.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "request-adapter" (func async (result u32)))
    (export "adapter-request-device" (func async (param "adapter" u32) (result u32)))
  ))
  (alias export $webgpu "request-adapter" (func $request-adapter))
  (alias export $webgpu "adapter-request-device" (func $adapter-request-device))

  (core module $m
    (import "" "request-adapter" (func $request-adapter (result i32)))
    (import "" "adapter-request-device" (func $adapter-request-device (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $adapter i32)
      (local.set $adapter (call $request-adapter))
      (call $adapter-request-device (local.get $adapter))
    )
  )
  (core func $ra_lower (canon lower (func $request-adapter)))
  (core func $rd_lower (canon lower (func $adapter-request-device)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "request-adapter" (func $ra_lower))
      (export "adapter-request-device" (func $rd_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
