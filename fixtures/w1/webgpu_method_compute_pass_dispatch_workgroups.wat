;; S6+: get-compute-pass + [method]gpu-compute-pass-encoder.dispatch-workgroups
;; WIT: dispatch-workgroups: func(x: u32, y: option<u32>, z: option<u32>)
;; Guest passes x=1, y=some(1), z=some(1); run returns harness 1.
;; L2 described JNI forwards pass rep + counts (options none → 1).
;; get-compute-pass is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $opt-u32 (option u32))
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $dispatch-ty (func
      (param "self" $borrow-pass)
      (param "workgroup-count-x" u32)
      (param "workgroup-count-y" $opt-u32)
      (param "workgroup-count-z" $opt-u32)))
    (export "[method]gpu-compute-pass-encoder.dispatch-workgroups" (func (type $dispatch-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.dispatch-workgroups" (func $dispatch))

  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $d_lower (canon lower (func $dispatch)))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "dispatch" (func $dispatch (param i32 i32 i32 i32 i32 i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local.set $pass (call $get-compute-pass))
      (call $dispatch
        (local.get $pass)
        (i32.const 1)
        (i32.const 1) (i32.const 1)
        (i32.const 1) (i32.const 1))
      (i32.const 1)
    )
  )
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
