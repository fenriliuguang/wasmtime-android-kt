;; S6+: get-texture-view + [method]gpu-texture-view.label
;; WIT: label: func() -> string. Host empty string; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-texture-view" (type $gpu-texture-view (sub resource)))
    (type $borrow-view (borrow $gpu-texture-view))
    (type $ty (func (param "self" $borrow-view) (result string)))
    (export "[method]gpu-texture-view.label" (func (type $ty)))
    (type $own-view (own $gpu-texture-view))
    (export "get-texture-view" (func (result $own-view)))
  ))
  (alias export $webgpu "get-texture-view" (func $get-view))
  (alias export $webgpu "[method]gpu-texture-view.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gv_lower (canon lower (func $get-view)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-view" (func $get-view (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-view) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-view" (func $gv_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
