;; S6+: get-compilation-message + [method]gpu-compilation-message.length
;; WIT: length: func() -> u64. Host returns 0; harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compilation-message" (type $gpu-compilation-message (sub resource)))
    (type $borrow-msg (borrow $gpu-compilation-message))
    (type $ty (func (param "self" $borrow-msg) (result u64)))
    (export "[method]gpu-compilation-message.length" (func (type $ty)))
    (type $own-msg (own $gpu-compilation-message))
    (export "get-compilation-message" (func (result $own-msg)))
  ))
  (alias export $webgpu "get-compilation-message" (func $get-msg))
  (alias export $webgpu "[method]gpu-compilation-message.length" (func $length))

  (core func $gm_lower (canon lower (func $get-msg)))
  (core func $l_lower (canon lower (func $length)))

  (core module $m
    (import "" "get-msg" (func $get-msg (result i32)))
    (import "" "length" (func $length (param i32) (result i64)))
    (func (export "run") (result i32)
      (drop (call $length (call $get-msg)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-msg" (func $gm_lower))
      (export "length" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
