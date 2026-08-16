;; W3+: get-compute-pass + [method]gpu-compute-pass-encoder.dispatch-workgroups
;; (self, x/y/z u32; void). Returns stub 79.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (export "get-compute-pass" (func (result (own $gpu-compute-pass-encoder))))
    (export "[method]gpu-compute-pass-encoder.dispatch-workgroups"
      (func (param "self" (borrow $gpu-compute-pass-encoder))
            (param "workgroup-count-x" u32)
            (param "workgroup-count-y" u32)
            (param "workgroup-count-z" u32)))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.dispatch-workgroups" (func $dispatch))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "dispatch" (func $dispatch (param i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-compute-pass))
      (call $dispatch (local.get $pass) (i32.const 1) (i32.const 1) (i32.const 1))
      (i32.const 79)
    )
  )
  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $d_lower (canon lower (func $dispatch)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "dispatch" (func $d_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
