;; W3+: get-pass + [method]gpu-render-pass-encoder.draw
;; (self, vertex-count u32; void). Returns stub 29.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (export "get-pass" (func (result (own $gpu-render-pass-encoder))))
    (export "[method]gpu-render-pass-encoder.draw"
      (func (param "self" (borrow $gpu-render-pass-encoder)) (param "vertex-count" u32)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.draw" (func $draw))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "draw" (func $draw (param i32 i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-pass))
      (call $draw (local.get $pass) (i32.const 3))
      (i32.const 29)
    )
  )
  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $d_lower (canon lower (func $draw)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "draw" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
