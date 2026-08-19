;; S6+: get-device-lost-info + [method]gpu-device-lost-info.message
;; WIT: message: func() -> string. Host empty string; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-device-lost-info" (type $gpu-device-lost-info (sub resource)))
    (type $borrow-info (borrow $gpu-device-lost-info))
    (type $ty (func (param "self" $borrow-info) (result string)))
    (export "[method]gpu-device-lost-info.message" (func (type $ty)))
    (type $own-info (own $gpu-device-lost-info))
    (export "get-device-lost-info" (func (result $own-info)))
  ))
  (alias export $webgpu "get-device-lost-info" (func $get-info))
  (alias export $webgpu "[method]gpu-device-lost-info.message" (func $message))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gi_lower (canon lower (func $get-info)))
  (core func $m_lower
    (canon lower (func $message)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-info" (func $get-info (result i32)))
    (import "" "message" (func $message (param i32 i32)))
    (func (export "run") (result i32)
      (call $message (call $get-info) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-info" (func $gi_lower))
      (export "message" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
