;; S6+: get-bind-group + [method]gpu-bind-group.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-bind-group is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-bind-group" (type $gpu-bind-group (sub resource)))
    (type $borrow-bg (borrow $gpu-bind-group))
    (type $ty (func (param "self" $borrow-bg) (result string)))
    (export "[method]gpu-bind-group.label" (func (type $ty)))
    (type $own-bg (own $gpu-bind-group))
    (export "get-bind-group" (func (result $own-bg)))
  ))
  (alias export $webgpu "get-bind-group" (func $get-bg))
  (alias export $webgpu "[method]gpu-bind-group.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gb_lower (canon lower (func $get-bg)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-bg" (func $get-bg (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-bg) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-bg" (func $gb_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
