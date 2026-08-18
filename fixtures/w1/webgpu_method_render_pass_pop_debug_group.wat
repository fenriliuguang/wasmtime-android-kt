;; S6+: get-pass + [method]gpu-render-pass-encoder.pop-debug-group
;; Guest constructs pass then pops; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $pop-ty (func (param "self" $borrow-pass)))
    (export "[method]gpu-render-pass-encoder.pop-debug-group" (func (type $pop-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.pop-debug-group" (func $pop))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $p_lower (canon lower (func $pop)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "pop" (func $pop (param i32)))
    (func (export "run") (result i32)
      (call $pop (call $get-pass))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "pop" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
