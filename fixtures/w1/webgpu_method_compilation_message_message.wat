;; S6+: get-compilation-message + [method]gpu-compilation-message.message
;; WIT: message: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-compilation-message is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compilation-message" (type $gpu-compilation-message (sub resource)))
    (type $borrow-msg (borrow $gpu-compilation-message))
    (type $ty (func (param "self" $borrow-msg) (result string)))
    (export "[method]gpu-compilation-message.message" (func (type $ty)))
    (type $own-msg (own $gpu-compilation-message))
    (export "get-compilation-message" (func (result $own-msg)))
  ))
  (alias export $webgpu "get-compilation-message" (func $get-msg))
  (alias export $webgpu "[method]gpu-compilation-message.message" (func $message))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gm_lower (canon lower (func $get-msg)))
  (core func $m_lower
    (canon lower (func $message)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-msg" (func $get-msg (result i32)))
    (import "" "message" (func $message (param i32 i32)))
    (func (export "run") (result i32)
      (call $message (call $get-msg) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-msg" (func $gm_lower))
      (export "message" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
