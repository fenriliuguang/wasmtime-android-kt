;; S6+: get-render-bundle-encoder + get-render-pipeline +
;; [method]gpu-render-bundle-encoder.set-pipeline
;; WIT: set-pipeline: func(pipeline: borrow<gpu-render-pipeline>)
;; Guest borrows the pipeline; drops owns; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pipeline" (type $gpu-render-pipeline (sub resource)))
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-pipe (borrow $gpu-render-pipeline))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $set-ty (func
      (param "self" $borrow-encoder)
      (param "pipeline" $borrow-pipe)))
    (export "[method]gpu-render-bundle-encoder.set-pipeline" (func (type $set-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
    (type $own-pipe (own $gpu-render-pipeline))
    (export "get-render-pipeline" (func (result $own-pipe)))
  ))
  (alias export $webgpu "gpu-render-pipeline" (type $gpu-render-pipeline))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "get-render-pipeline" (func $get-render-pipeline))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.set-pipeline" (func $set-pipeline))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $gpipe_lower (canon lower (func $get-render-pipeline)))
  (core func $sp_lower (canon lower (func $set-pipeline)))
  (core func $dpipe_lower (canon resource.drop $gpu-render-pipeline))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "get-render-pipeline" (func $get-render-pipeline (result i32)))
    (import "" "set-pipeline" (func $set-pipeline (param i32 i32)))
    (import "" "drop-pipeline" (func $drop-pipeline (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $pipe i32)
      (local.set $encoder (call $get-encoder))
      (local.set $pipe (call $get-render-pipeline))
      (call $set-pipeline (local.get $encoder) (local.get $pipe))
      (call $drop-pipeline (local.get $pipe))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "get-render-pipeline" (func $gpipe_lower))
      (export "set-pipeline" (func $sp_lower))
      (export "drop-pipeline" (func $dpipe_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
