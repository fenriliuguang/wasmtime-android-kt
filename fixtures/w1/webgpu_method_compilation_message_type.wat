;; S6+: get-compilation-message + [method]gpu-compilation-message.type
;; WIT: type: func() -> gpu-compilation-message-type. Host returns error; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $mt (enum "error" "warning" "info"))
    (export "gpu-compilation-message-type" (type $gpu-compilation-message-type (eq $mt)))
    (export "gpu-compilation-message" (type $gpu-compilation-message (sub resource)))
    (type $borrow-msg (borrow $gpu-compilation-message))
    (type $ty-fn (func (param "self" $borrow-msg) (result $gpu-compilation-message-type)))
    (export "[method]gpu-compilation-message.type" (func (type $ty-fn)))
    (type $own-msg (own $gpu-compilation-message))
    (export "get-compilation-message" (func (result $own-msg)))
  ))
  (alias export $webgpu "get-compilation-message" (func $get-msg))
  (alias export $webgpu "[method]gpu-compilation-message.type" (func $ty))

  (core func $gm_lower (canon lower (func $get-msg)))
  (core func $t_lower (canon lower (func $ty)))

  (core module $m
    (import "" "get-msg" (func $get-msg (result i32)))
    (import "" "ty" (func $ty (param i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $ty (call $get-msg)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-msg" (func $gm_lower))
      (export "ty" (func $t_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
