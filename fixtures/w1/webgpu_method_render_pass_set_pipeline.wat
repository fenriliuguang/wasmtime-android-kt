;; S6+: get-pass + get-render-pipeline +
;; [method]gpu-render-pass-encoder.set-pipeline
;; WIT: set-pipeline: func(pipeline: borrow<gpu-render-pipeline>)
;; Guest borrows the pipeline; drops owns; run returns harness 1.
;; L2 still host-fixed triangle pipeline.
;; get-pass / get-render-pipeline are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pipeline" (type $gpu-render-pipeline (sub resource)))
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pipe (borrow $gpu-render-pipeline))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $set-ty (func
      (param "self" $borrow-pass)
      (param "pipeline" $borrow-pipe)))
    (export "[method]gpu-render-pass-encoder.set-pipeline" (func (type $set-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
    (type $own-pipe (own $gpu-render-pipeline))
    (export "get-render-pipeline" (func (result $own-pipe)))
  ))
  (alias export $webgpu "gpu-render-pipeline" (type $gpu-render-pipeline))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "get-render-pipeline" (func $get-render-pipeline))
  (alias export $webgpu "[method]gpu-render-pass-encoder.set-pipeline" (func $set-pipeline))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $gpipe_lower (canon lower (func $get-render-pipeline)))
  (core func $sp_lower (canon lower (func $set-pipeline)))
  (core func $dpipe_lower (canon resource.drop $gpu-render-pipeline))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "get-render-pipeline" (func $get-render-pipeline (result i32)))
    (import "" "set-pipeline" (func $set-pipeline (param i32 i32)))
    (import "" "drop-pipeline" (func $drop-pipeline (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local $pipe i32)
      (local.set $pass (call $get-pass))
      (local.set $pipe (call $get-render-pipeline))
      (call $set-pipeline (local.get $pass) (local.get $pipe))
      (call $drop-pipeline (local.get $pipe))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "get-render-pipeline" (func $gpipe_lower))
      (export "set-pipeline" (func $sp_lower))
      (export "drop-pipeline" (func $dpipe_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
