;; S6+: get-render-bundle-encoder + [method]gpu-render-bundle-encoder.pop-debug-group
;; Guest constructs encoder then pops; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $pop-ty (func (param "self" $borrow-encoder)))
    (export "[method]gpu-render-bundle-encoder.pop-debug-group" (func (type $pop-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.pop-debug-group" (func $pop))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $p_lower (canon lower (func $pop)))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "pop" (func $pop (param i32)))
    (func (export "run") (result i32)
      (call $pop (call $get-encoder))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "pop" (func $p_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
