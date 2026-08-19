;; S6+: get-device + [method]gpu-device.set-label
;; Guest passes empty label; run returns harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-dev (borrow $gpu-device))
    (type $set-ty (func
      (param "self" $borrow-dev)
      (param "label" string)))
    (export "[method]gpu-device.set-label" (func (type $set-ty)))
    (type $own-dev (own $gpu-device))
    (export "get-device" (func (result $own-dev)))
  ))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-device) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-device" (func $gd_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
