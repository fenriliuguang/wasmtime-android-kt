;; S6+: get-device + [method]gpu-device.adapter-info
;; WIT: adapter-info: func() -> gpu-adapter-info. Drop own; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "gpu-adapter-info" (type $gpu-adapter-info (sub resource)))
    (type $borrow-dev (borrow $gpu-device))
    (type $own-info (own $gpu-adapter-info))
    (type $ty (func (param "self" $borrow-dev) (result $own-info)))
    (export "[method]gpu-device.adapter-info" (func (type $ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "gpu-adapter-info" (type $gpu-adapter-info))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.adapter-info" (func $adapter-info))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $ai_lower (canon lower (func $adapter-info)))
  (core func $df_lower (canon resource.drop $gpu-adapter-info))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "adapter-info" (func $adapter-info (param i32) (result i32)))
    (import "" "drop-info" (func $drop-info (param i32)))
    (func (export "run") (result i32)
      (call $drop-info (call $adapter-info (call $get-device)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "adapter-info" (func $ai_lower))
      (export "drop-info" (func $df_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
