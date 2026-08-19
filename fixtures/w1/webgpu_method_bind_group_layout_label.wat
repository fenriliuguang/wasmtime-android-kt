;; S6+: get-bind-group-layout + [method]gpu-bind-group-layout.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-bind-group-layout is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-bind-group-layout" (type $gpu-bind-group-layout (sub resource)))
    (type $borrow-layout (borrow $gpu-bind-group-layout))
    (type $ty (func (param "self" $borrow-layout) (result string)))
    (export "[method]gpu-bind-group-layout.label" (func (type $ty)))
    (type $own-layout (own $gpu-bind-group-layout))
    (export "get-bind-group-layout" (func (result $own-layout)))
  ))
  (alias export $webgpu "get-bind-group-layout" (func $get-layout))
  (alias export $webgpu "[method]gpu-bind-group-layout.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gl_lower (canon lower (func $get-layout)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-layout" (func $get-layout (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-layout) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-layout" (func $gl_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
