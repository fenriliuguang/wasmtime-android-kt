;; S6+: get-queue + [method]gpu-queue.set-label
;; Guest passes empty label; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (type $borrow-q (borrow $gpu-queue))
    (type $set-ty (func
      (param "self" $borrow-q)
      (param "label" string)))
    (export "[method]gpu-queue.set-label" (func (type $set-ty)))
    (type $own-q (own $gpu-queue))
    (export "get-queue" (func (result $own-q)))
  ))
  (alias export $webgpu "get-queue" (func $get-q))
  (alias export $webgpu "[method]gpu-queue.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gq_lower (canon lower (func $get-q)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-q" (func $get-q (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-q) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-q" (func $gq_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
