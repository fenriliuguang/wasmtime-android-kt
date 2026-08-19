;; S6+: get-device + [method]gpu-device.limits
;; WIT: limits: func() -> gpu-supported-limits. Drop own; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "gpu-supported-limits" (type $gpu-supported-limits (sub resource)))
    (type $borrow-dev (borrow $gpu-device))
    (type $own-lim (own $gpu-supported-limits))
    (type $ty (func (param "self" $borrow-dev) (result $own-lim)))
    (export "[method]gpu-device.limits" (func (type $ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "gpu-supported-limits" (type $gpu-supported-limits))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.limits" (func $limits))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $l_lower (canon lower (func $limits)))
  (core func $df_lower (canon resource.drop $gpu-supported-limits))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "limits" (func $limits (param i32) (result i32)))
    (import "" "drop-lim" (func $drop-lim (param i32)))
    (func (export "run") (result i32)
      (call $drop-lim (call $limits (call $get-device)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "limits" (func $l_lower))
      (export "drop-lim" (func $df_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
