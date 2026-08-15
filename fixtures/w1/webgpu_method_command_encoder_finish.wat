;; W3: get-encoder + [method]gpu-command-encoder.finish (sync u32).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (export "get-encoder" (func (result (own $gpu-command-encoder))))
    (export "[method]gpu-command-encoder.finish"
      (func (param "self" (borrow $gpu-command-encoder)) (result u32)))
  ))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.finish" (func $finish))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "finish" (func $finish (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local.set $encoder (call $get-encoder))
      (call $finish (local.get $encoder))
    )
  )
  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $f_lower (canon lower (func $finish)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "finish" (func $f_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
