;; S6+: get-pass + [method]gpu-render-pass-encoder.label
;; WIT: label: func() -> string. Host empty string; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-res (borrow $gpu-render-pass-encoder))
    (type $ty (func (param "self" $borrow-res) (result string)))
    (export "[method]gpu-render-pass-encoder.label" (func (type $ty)))
    (type $own-res (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-res)))
  ))
  (alias export $webgpu "get-pass" (func $get-res))
  (alias export $webgpu "[method]gpu-render-pass-encoder.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gr_lower (canon lower (func $get-res)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-res" (func $get-res (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-res) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-res" (func $gr_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
