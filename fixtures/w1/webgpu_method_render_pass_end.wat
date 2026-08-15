;; W3: get-pass + [method]gpu-render-pass-encoder.end (void). Returns stub 29.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (export "get-pass" (func (result (own $gpu-render-pass-encoder))))
    (export "[method]gpu-render-pass-encoder.end"
      (func (param "self" (borrow $gpu-render-pass-encoder))))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.end" (func $end))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "end" (func $end (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-pass))
      (call $end (local.get $pass))
      (i32.const 29)
    )
  )
  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $e_lower (canon lower (func $end)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "end" (func $e_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
