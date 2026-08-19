;; S6+: get-device + [method]gpu-device.destroy
;; WIT: destroy: func(). Guest constructs device then destroys; harness 1.
;; L2 unused (lift-only). get-device is a test constructor (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $destroy-ty (func (param "self" $borrow-device)))
    (export "[method]gpu-device.destroy" (func (type $destroy-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.destroy" (func $destroy))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $d_lower (canon lower (func $destroy)))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "destroy" (func $destroy (param i32)))
    (func (export "run") (result i32)
      (call $destroy (call $get-device))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "destroy" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
