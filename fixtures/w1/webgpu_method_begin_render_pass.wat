;; W3: get-encoder + [method]gpu-command-encoder.begin-render-pass
;; (resource self; sync; transitional view u32, stub 23). Flat
;; command-encoder-begin-render-pass-clear stays registered.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (export "get-encoder" (func (result (own $gpu-command-encoder))))
    (export "[method]gpu-command-encoder.begin-render-pass"
      (func (param "self" (borrow $gpu-command-encoder)) (param "view" u32) (result u32)))
  ))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.begin-render-pass" (func $begin))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "begin" (func $begin (param i32 i32) (result i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local.set $encoder (call $get-encoder))
      (call $begin (local.get $encoder) (i32.const 23))
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
