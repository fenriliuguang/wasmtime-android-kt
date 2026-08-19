;; S6+: get-buffer + [method]gpu-buffer.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-buffer is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (type $borrow-buf (borrow $gpu-buffer))
    (type $ty (func (param "self" $borrow-buf) (result string)))
    (export "[method]gpu-buffer.label" (func (type $ty)))
    (type $own-buf (own $gpu-buffer))
    (export "get-buffer" (func (result $own-buf)))
  ))
  (alias export $webgpu "get-buffer" (func $get-buf))
  (alias export $webgpu "[method]gpu-buffer.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gb_lower (canon lower (func $get-buf)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-buf" (func $get-buf (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-buf) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-buf" (func $gb_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
