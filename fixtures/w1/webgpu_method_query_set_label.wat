;; S6+: get-query-set + [method]gpu-query-set.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-query-set is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (type $borrow-qs (borrow $gpu-query-set))
    (type $ty (func (param "self" $borrow-qs) (result string)))
    (export "[method]gpu-query-set.label" (func (type $ty)))
    (type $own-qs (own $gpu-query-set))
    (export "get-query-set" (func (result $own-qs)))
  ))
  (alias export $webgpu "get-query-set" (func $get-qs))
  (alias export $webgpu "[method]gpu-query-set.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gq_lower (canon lower (func $get-qs)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-qs" (func $get-qs (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-qs) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-qs" (func $gq_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
