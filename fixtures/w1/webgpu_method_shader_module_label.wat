;; S6+: get-shader-module + [method]gpu-shader-module.label
;; WIT: label: func() -> string. Host empty string; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-shader-module" (type $gpu-shader-module (sub resource)))
    (type $borrow-shader (borrow $gpu-shader-module))
    (type $ty (func (param "self" $borrow-shader) (result string)))
    (export "[method]gpu-shader-module.label" (func (type $ty)))
    (type $own-shader (own $gpu-shader-module))
    (export "get-shader-module" (func (result $own-shader)))
  ))
  (alias export $webgpu "get-shader-module" (func $get-shader))
  (alias export $webgpu "[method]gpu-shader-module.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gs_lower (canon lower (func $get-shader)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-shader" (func $get-shader (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-shader) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-shader" (func $gs_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
