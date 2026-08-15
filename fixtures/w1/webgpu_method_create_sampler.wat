;; W3+: wasi:webgpu/webgpu@0.3.0-rc.2 get-device +
;; [method]gpu-device.create-sampler (resource self; sync).
;; Transitional: method returns u32 (host-fixed default descriptor).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "get-device" (func (result (own $gpu-device))))
    (export "[method]gpu-device.create-sampler"
      (func (param "self" (borrow $gpu-device)) (result u32)))
  ))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-sampler" (func $create-sampler))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-sampler" (func $create-sampler (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local.set $device (call $get-device))
      (call $create-sampler (local.get $device))
    )
  )
  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cs_lower (canon lower (func $create-sampler)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "create-sampler" (func $cs_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
