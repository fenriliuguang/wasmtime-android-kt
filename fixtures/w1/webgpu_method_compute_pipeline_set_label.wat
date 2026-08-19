;; S6+: get-compute-pipeline + [method]gpu-compute-pipeline.set-label
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pipeline" (type $gpu-compute-pipeline (sub resource)))
    (type $borrow-pipeline (borrow $gpu-compute-pipeline))
    (type $set-ty (func
      (param "self" $borrow-pipeline)
      (param "label" string)))
    (export "[method]gpu-compute-pipeline.set-label" (func (type $set-ty)))
    (type $own-pipeline (own $gpu-compute-pipeline))
    (export "get-compute-pipeline" (func (result $own-pipeline)))
  ))
  (alias export $webgpu "get-compute-pipeline" (func $get-pipeline))
  (alias export $webgpu "[method]gpu-compute-pipeline.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gcp_lower (canon lower (func $get-pipeline)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-pipeline" (func $get-pipeline (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-pipeline) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pipeline" (func $gcp_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
