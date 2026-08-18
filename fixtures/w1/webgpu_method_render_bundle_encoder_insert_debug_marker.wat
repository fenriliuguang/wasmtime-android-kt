;; S6+: get-render-bundle-encoder + [method]gpu-render-bundle-encoder.insert-debug-marker
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $insert-ty (func
      (param "self" $borrow-encoder)
      (param "marker-label" string)))
    (export "[method]gpu-render-bundle-encoder.insert-debug-marker" (func (type $insert-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.insert-debug-marker" (func $insert))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $i_lower
    (canon lower (func $insert)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "insert" (func $insert (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $insert (call $get-encoder) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "insert" (func $i_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
