;; S6+: get-encoder + [method]gpu-command-encoder.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-encoder is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-enc (borrow $gpu-command-encoder))
    (type $ty (func (param "self" $borrow-enc) (result string)))
    (export "[method]gpu-command-encoder.label" (func (type $ty)))
    (type $own-enc (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-enc)))
  ))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-encoder) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-encoder" (func $ge_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
