;; S6+: get-queue + [method]gpu-queue.on-submitted-work-done
;; WIT: async on-submitted-work-done: func(). Host no-op; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (type $borrow-q (borrow $gpu-queue))
    (type $done-ty (func async (param "self" $borrow-q)))
    (export "[method]gpu-queue.on-submitted-work-done" (func (type $done-ty)))
    (type $own-q (own $gpu-queue))
    (export "get-queue" (func (result $own-q)))
  ))
  (alias export $webgpu "get-queue" (func $get-queue))
  (alias export $webgpu "[method]gpu-queue.on-submitted-work-done" (func $done))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gq_lower (canon lower (func $get-queue)))
  (core func $done_lower
    (canon lower (func $done)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-queue" (func $get-queue (result i32)))
    (import "" "done" (func $done (param i32)))
    (func (export "run") (result i32)
      (call $done (call $get-queue))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-queue" (func $gq_lower))
      (export "done" (func $done_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
