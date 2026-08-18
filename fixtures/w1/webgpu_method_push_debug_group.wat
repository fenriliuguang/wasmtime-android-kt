;; S6+: get-encoder + [method]gpu-command-encoder.push-debug-group
;; WIT: push-debug-group: func(group-label: string)
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-enc (borrow $gpu-command-encoder))
    (type $push-ty (func
      (param "self" $borrow-enc)
      (param "group-label" string)))
    (export "[method]gpu-command-encoder.push-debug-group" (func (type $push-ty)))
    (type $own-enc (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-enc)))
  ))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.push-debug-group" (func $push))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $p_lower
    (canon lower (func $push)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "push" (func $push (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $push (call $get-encoder) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "push" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
