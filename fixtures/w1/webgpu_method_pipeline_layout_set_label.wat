;; S6+: get-pipeline-layout + [method]gpu-pipeline-layout.set-label
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-pipeline-layout" (type $gpu-pipeline-layout (sub resource)))
    (type $borrow-pl (borrow $gpu-pipeline-layout))
    (type $set-ty (func
      (param "self" $borrow-pl)
      (param "label" string)))
    (export "[method]gpu-pipeline-layout.set-label" (func (type $set-ty)))
    (type $own-pl (own $gpu-pipeline-layout))
    (export "get-pipeline-layout" (func (result $own-pl)))
  ))
  (alias export $webgpu "get-pipeline-layout" (func $get-pl))
  (alias export $webgpu "[method]gpu-pipeline-layout.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gpl_lower (canon lower (func $get-pl)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-pl" (func $get-pl (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-pl) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pl" (func $gpl_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
