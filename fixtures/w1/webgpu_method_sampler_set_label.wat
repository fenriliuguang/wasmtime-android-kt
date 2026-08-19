;; S6+: get-sampler + [method]gpu-sampler.set-label
;; WIT: set-label: func(label: string). Guest passes empty label; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-sampler" (type $gpu-sampler (sub resource)))
    (type $borrow-sampler (borrow $gpu-sampler))
    (type $ty (func (param "self" $borrow-sampler) (param "label" string)))
    (export "[method]gpu-sampler.set-label" (func (type $ty)))
    (type $own-sampler (own $gpu-sampler))
    (export "get-sampler" (func (result $own-sampler)))
  ))
  (alias export $webgpu "get-sampler" (func $get-sampler))
  (alias export $webgpu "[method]gpu-sampler.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gs_lower (canon lower (func $get-sampler)))
  (core func $sl_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-sampler" (func $get-sampler (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-sampler) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-sampler" (func $gs_lower))
      (export "set-label" (func $sl_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
