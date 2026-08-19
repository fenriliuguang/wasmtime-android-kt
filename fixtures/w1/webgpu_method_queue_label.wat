;; S6+: get-queue + [method]gpu-queue.label
;; WIT: label: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-queue is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (type $borrow-q (borrow $gpu-queue))
    (type $ty (func (param "self" $borrow-q) (result string)))
    (export "[method]gpu-queue.label" (func (type $ty)))
    (type $own-q (own $gpu-queue))
    (export "get-queue" (func (result $own-q)))
  ))
  (alias export $webgpu "get-queue" (func $get-q))
  (alias export $webgpu "[method]gpu-queue.label" (func $label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gq_lower (canon lower (func $get-q)))
  (core func $l_lower
    (canon lower (func $label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-q" (func $get-q (result i32)))
    (import "" "label" (func $label (param i32 i32)))
    (func (export "run") (result i32)
      (call $label (call $get-q) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-q" (func $gq_lower))
      (export "label" (func $l_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
