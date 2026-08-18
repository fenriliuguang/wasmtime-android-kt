;; S6+: get-pass + [method]gpu-render-pass-encoder.end-occlusion-query
;; Guest constructs pass then ends query; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $end-ty (func (param "self" $borrow-pass)))
    (export "[method]gpu-render-pass-encoder.end-occlusion-query" (func (type $end-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.end-occlusion-query" (func $end))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $e_lower (canon lower (func $end)))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "end" (func $end (param i32)))
    (func (export "run") (result i32)
      (call $end (call $get-pass))
      (i32.const 1)
    )
  )
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
