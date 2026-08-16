;; W3+: get-compute-pass + [method]gpu-compute-pass-encoder.set-pipeline
;; (self, pipeline u32; void). Returns stub 73.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (export "get-compute-pass" (func (result (own $gpu-compute-pass-encoder))))
    (export "[method]gpu-compute-pass-encoder.set-pipeline"
      (func (param "self" (borrow $gpu-compute-pass-encoder)) (param "pipeline" u32)))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.set-pipeline" (func $set-pipeline))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "set-pipeline" (func $set-pipeline (param i32 i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-compute-pass))
      (call $set-pipeline (local.get $pass) (i32.const 73))
      (i32.const 73)
    )
  )
  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $sp_lower (canon lower (func $set-pipeline)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "set-pipeline" (func $sp_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
