;; S6+: get-gpu-error + [method]gpu-error.message
;; L2 described device handle → message (Cpu stub cpu-gpu-error); harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-error" (type $gpu-error (sub resource)))
    (type $borrow-err (borrow $gpu-error))
    (type $ty (func (param "self" $borrow-err) (result string)))
    (export "[method]gpu-error.message" (func (type $ty)))
    (type $own-err (own $gpu-error))
    (export "get-gpu-error" (func (result $own-err)))
  ))
  (alias export $webgpu "get-gpu-error" (func $get-err))
  (alias export $webgpu "[method]gpu-error.message" (func $message))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-err)))
  (core func $m_lower
    (canon lower (func $message)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-err" (func $get-err (result i32)))
    (import "" "message" (func $message (param i32 i32)))
    (func (export "run") (result i32)
      (call $message (call $get-err) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-err" (func $ge_lower))
      (export "message" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
