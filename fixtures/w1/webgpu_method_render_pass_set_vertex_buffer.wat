;; W3+: get-pass + [method]gpu-render-pass-encoder.set-vertex-buffer
;; (self, buffer u32; void). Returns stub 31.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (export "get-pass" (func (result (own $gpu-render-pass-encoder))))
    (export "[method]gpu-render-pass-encoder.set-vertex-buffer"
      (func (param "self" (borrow $gpu-render-pass-encoder)) (param "buffer" u32)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.set-vertex-buffer" (func $set-vertex-buffer))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "set-vertex-buffer" (func $set-vertex-buffer (param i32 i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-pass))
      (call $set-vertex-buffer (local.get $pass) (i32.const 31))
      (i32.const 31)
    )
  )
  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $svb_lower (canon lower (func $set-vertex-buffer)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "set-vertex-buffer" (func $svb_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
