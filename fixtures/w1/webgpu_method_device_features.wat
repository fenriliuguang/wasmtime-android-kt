;; S6+: get-device + [method]gpu-device.features
;; WIT: features: func() -> gpu-supported-features. Drop own; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (export "gpu-supported-features" (type $gpu-supported-features (sub resource)))
    (type $borrow-dev (borrow $gpu-device))
    (type $own-feat (own $gpu-supported-features))
    (type $ty (func (param "self" $borrow-dev) (result $own-feat)))
    (export "[method]gpu-device.features" (func (type $ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "gpu-supported-features" (type $gpu-supported-features))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.features" (func $features))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $f_lower (canon lower (func $features)))
  (core func $df_lower (canon resource.drop $gpu-supported-features))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "features" (func $features (param i32) (result i32)))
    (import "" "drop-feat" (func $drop-feat (param i32)))
    (func (export "run") (result i32)
      (call $drop-feat (call $features (call $get-device)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "features" (func $f_lower))
      (export "drop-feat" (func $df_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
