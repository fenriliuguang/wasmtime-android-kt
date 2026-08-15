;; W3: wasi:webgpu/webgpu@0.3.0-rc.2 get-adapter + [method]gpu-adapter.request-device
;; (resource self; true CM async). Transitional: method returns u32 (not
;; result<gpu-device, request-device-error>); no descriptor. Flat
;; adapter-request-device stays registered.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter" (type $gpu-adapter (sub resource)))
    (export "get-adapter" (func (result (own $gpu-adapter))))
    (export "[method]gpu-adapter.request-device"
      (func async (param "self" (borrow $gpu-adapter)) (result u32)))
  ))
  (alias export $webgpu "get-adapter" (func $get-adapter))
  (alias export $webgpu "[method]gpu-adapter.request-device" (func $request-device))

  (core module $m
    (import "" "get-adapter" (func $get-adapter (result i32)))
    (import "" "request-device" (func $request-device (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $adapter i32)
      (local.set $adapter (call $get-adapter))
      (call $request-device (local.get $adapter))
    )
  )
  (core func $ga_lower (canon lower (func $get-adapter)))
  (core func $rd_lower (canon lower (func $request-device)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-adapter" (func $ga_lower))
      (export "request-device" (func $rd_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
