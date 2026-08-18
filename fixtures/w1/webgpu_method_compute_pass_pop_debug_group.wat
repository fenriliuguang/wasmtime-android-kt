;; S6+: get-compute-pass + [method]gpu-compute-pass-encoder.pop-debug-group
;; Guest constructs compute-pass then pops; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $pop-ty (func (param "self" $borrow-pass)))
    (export "[method]gpu-compute-pass-encoder.pop-debug-group" (func (type $pop-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.pop-debug-group" (func $pop))

  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $p_lower (canon lower (func $pop)))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "pop" (func $pop (param i32)))
    (func (export "run") (result i32)
      (call $pop (call $get-compute-pass))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "pop" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
