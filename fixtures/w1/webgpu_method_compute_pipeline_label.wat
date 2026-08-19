;; S6+: get-compute-pipeline + [method]gpu-compute-pipeline.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-compute-pipeline is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pipeline" (type $gpu-compute-pipeline (sub resource)))
    (type $borrow-pipeline (borrow $gpu-compute-pipeline))
    (type $ty (func (param "self" $borrow-pipeline) (result string)))
    (export "[method]gpu-compute-pipeline.label" (func (type $ty)))
    (type $own-pipeline (own $gpu-compute-pipeline))
    (export "get-compute-pipeline" (func (result $own-pipeline)))
  ))
  (alias export $webgpu "get-compute-pipeline" (func $get-pipeline))
  (alias export $webgpu "[method]gpu-compute-pipeline.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gcp_lower (canon lower (func $get-pipeline)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-pipeline" (func $get-pipeline (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-pipeline) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-pipeline" (func $gcp_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
