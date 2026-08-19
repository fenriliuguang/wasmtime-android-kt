;; S6+: get-shader-module + [method]gpu-shader-module.get-compilation-info
;; WIT: async get-compilation-info: func() -> gpu-compilation-info. Host empty info; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-shader-module" (type $gpu-shader-module (sub resource)))
    (export "gpu-compilation-info" (type $gpu-compilation-info (sub resource)))
    (type $borrow-shader (borrow $gpu-shader-module))
    (type $own-info (own $gpu-compilation-info))
    (type $get-ty (func async (param "self" $borrow-shader) (result $own-info)))
    (export "[method]gpu-shader-module.get-compilation-info" (func (type $get-ty)))
    (type $own-shader (own $gpu-shader-module))
    (export "get-shader-module" (func (result $own-shader)))
  ))
  (alias export $webgpu "gpu-compilation-info" (type $gpu-compilation-info))
  (alias export $webgpu "get-shader-module" (func $get-shader))
  (alias export $webgpu "[method]gpu-shader-module.get-compilation-info" (func $get-info))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gs_lower (canon lower (func $get-shader)))
  (core func $gi_lower
    (canon lower (func $get-info)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $di_lower (canon resource.drop $gpu-compilation-info))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-shader" (func $get-shader (result i32)))
    (import "" "get-info" (func $get-info (param i32) (result i32)))
    (import "" "drop-info" (func $drop-info (param i32)))
    (func (export "run") (result i32)
      (call $drop-info (call $get-info (call $get-shader)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-shader" (func $gs_lower))
      (export "get-info" (func $gi_lower))
      (export "drop-info" (func $di_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
