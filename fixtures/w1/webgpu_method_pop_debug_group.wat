;; S6+: get-encoder + [method]gpu-command-encoder.pop-debug-group
;; WIT: pop-debug-group: func()
;; Guest constructs encoder then pops; run returns harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-enc (borrow $gpu-command-encoder))
    (type $pop-ty (func (param "self" $borrow-enc)))
    (export "[method]gpu-command-encoder.pop-debug-group" (func (type $pop-ty)))
    (type $own-enc (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-enc)))
  ))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.pop-debug-group" (func $pop))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $p_lower (canon lower (func $pop)))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "pop" (func $pop (param i32)))
    (func (export "run") (result i32)
      (call $pop (call $get-encoder))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "pop" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
