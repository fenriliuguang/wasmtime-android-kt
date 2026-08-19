;; S6+: get-compilation-message + [method]gpu-compilation-message.offset
;; WIT: offset: func() -> u64. Host returns 0; harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compilation-message" (type $gpu-compilation-message (sub resource)))
    (type $borrow-msg (borrow $gpu-compilation-message))
    (type $ty (func (param "self" $borrow-msg) (result u64)))
    (export "[method]gpu-compilation-message.offset" (func (type $ty)))
    (type $own-msg (own $gpu-compilation-message))
    (export "get-compilation-message" (func (result $own-msg)))
  ))
  (alias export $webgpu "get-compilation-message" (func $get-msg))
  (alias export $webgpu "[method]gpu-compilation-message.offset" (func $offset))

  (core func $gm_lower (canon lower (func $get-msg)))
  (core func $o_lower (canon lower (func $offset)))

  (core module $m
    (import "" "get-msg" (func $get-msg (result i32)))
    (import "" "offset" (func $offset (param i32) (result i64)))
    (func (export "run") (result i32)
      (drop (call $offset (call $get-msg)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-msg" (func $gm_lower))
      (export "offset" (func $o_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
