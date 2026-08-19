;; S6+: get-compute-pass + [method]gpu-compute-pass-encoder.set-label
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $set-ty (func
      (param "self" $borrow-pass)
      (param "label" string)))
    (export "[method]gpu-compute-pass-encoder.set-label" (func (type $set-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-pass) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
