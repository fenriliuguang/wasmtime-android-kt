;; S6+: get-pipeline-layout + [method]gpu-pipeline-layout.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-pipeline-layout is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-pipeline-layout" (type $gpu-pipeline-layout (sub resource)))
    (type $borrow-pl (borrow $gpu-pipeline-layout))
    (type $ty (func (param "self" $borrow-pl) (result string)))
    (export "[method]gpu-pipeline-layout.label" (func (type $ty)))
    (type $own-pl (own $gpu-pipeline-layout))
    (export "get-pipeline-layout" (func (result $own-pl)))
  ))
  (alias export $webgpu "get-pipeline-layout" (func $get-pl))
  (alias export $webgpu "[method]gpu-pipeline-layout.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gpl_lower (canon lower (func $get-pl)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-pl" (func $get-pl (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-pl) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-pl" (func $gpl_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
