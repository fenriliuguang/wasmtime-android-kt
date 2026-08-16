;; W3+: get-encoder + [method]gpu-command-encoder.begin-compute-pass
;; (resource self; sync; no descriptor; transitional pass u32, stub 79).
;; Flat names stay registered.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (export "get-encoder" (func (result (own $gpu-command-encoder))))
    (export "[method]gpu-command-encoder.begin-compute-pass"
      (func (param "self" (borrow $gpu-command-encoder)) (result u32)))
  ))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.begin-compute-pass" (func $begin))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "begin" (func $begin (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local.set $encoder (call $get-encoder))
      (call $begin (local.get $encoder))
    )
  )
  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $b_lower (canon lower (func $begin)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "begin" (func $b_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
