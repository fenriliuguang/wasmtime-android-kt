;; S6+: get-pass + [method]gpu-render-pass-encoder.push-debug-group
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $push-ty (func
      (param "self" $borrow-pass)
      (param "group-label" string)))
    (export "[method]gpu-render-pass-encoder.push-debug-group" (func (type $push-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.push-debug-group" (func $push))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $p_lower
    (canon lower (func $push)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "push" (func $push (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $push (call $get-pass) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "push" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
