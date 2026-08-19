;; S6+: get-sampler + [method]gpu-sampler.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-sampler is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-sampler" (type $gpu-sampler (sub resource)))
    (type $borrow-sampler (borrow $gpu-sampler))
    (type $ty (func (param "self" $borrow-sampler) (result string)))
    (export "[method]gpu-sampler.label" (func (type $ty)))
    (type $own-sampler (own $gpu-sampler))
    (export "get-sampler" (func (result $own-sampler)))
  ))
  (alias export $webgpu "get-sampler" (func $get-sampler))
  (alias export $webgpu "[method]gpu-sampler.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gs_lower (canon lower (func $get-sampler)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-sampler" (func $get-sampler (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-sampler) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-sampler" (func $gs_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
