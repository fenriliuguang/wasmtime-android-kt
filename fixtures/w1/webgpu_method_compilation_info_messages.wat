;; S6+: get-compilation-info + [method]gpu-compilation-info.messages
;; WIT: messages: func() -> list<gpu-compilation-message>. Host empty list; harness 1.
;; L2 unused (lift-only). get-compilation-info is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compilation-message" (type $gpu-compilation-message (sub resource)))
    (export "gpu-compilation-info" (type $gpu-compilation-info (sub resource)))
    (type $own-msg (own $gpu-compilation-message))
    (type $list-msg (list $own-msg))
    (type $borrow-info (borrow $gpu-compilation-info))
    (type $msgs-ty (func (param "self" $borrow-info) (result $list-msg)))
    (export "[method]gpu-compilation-info.messages" (func (type $msgs-ty)))
    (type $own-info (own $gpu-compilation-info))
    (export "get-compilation-info" (func (result $own-info)))
  ))
  (alias export $webgpu "get-compilation-info" (func $get-info))
  (alias export $webgpu "[method]gpu-compilation-info.messages" (func $messages))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gi_lower (canon lower (func $get-info)))
  (core func $m_lower
    (canon lower (func $messages)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-info" (func $get-info (result i32)))
    (import "" "messages" (func $messages (param i32 i32)))
    (func (export "run") (result i32)
      (call $messages (call $get-info) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-info" (func $gi_lower))
      (export "messages" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
