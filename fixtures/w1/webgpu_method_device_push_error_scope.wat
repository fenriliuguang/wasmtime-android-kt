;; S6+: get-device + [method]gpu-device.push-error-scope
;; WIT: push-error-scope: func(filter: gpu-error-filter). Guest validation; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $filter (enum "validation" "out-of-memory" "internal"))
    (export "gpu-error-filter" (type $gpu-error-filter (eq $filter)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-dev (borrow $gpu-device))
    (type $ty (func (param "self" $borrow-dev) (param "filter" $gpu-error-filter)))
    (export "[method]gpu-device.push-error-scope" (func (type $ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.push-error-scope" (func $push))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $p_lower (canon lower (func $push)))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "push" (func $push (param i32 i32)))
    (func (export "run") (result i32)
      (call $push (call $get-device) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "push" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
