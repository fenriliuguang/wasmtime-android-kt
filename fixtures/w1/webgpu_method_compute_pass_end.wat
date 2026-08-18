;; W3+: get-compute-pass + [method]gpu-compute-pass-encoder.end (void).
;; Guest calls end then returns harness 1. Do not reuse get-pass (that is render-pass).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (export "get-compute-pass" (func (result (own $gpu-compute-pass-encoder))))
    (export "[method]gpu-compute-pass-encoder.end"
      (func (param "self" (borrow $gpu-compute-pass-encoder))))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.end" (func $end))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "end" (func $end (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-compute-pass))
      (call $end (local.get $pass))
      (i32.const 1)
    )
  )
  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $e_lower (canon lower (func $end)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "end" (func $e_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
