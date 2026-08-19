;; S6+: get-compute-pass + get-compute-pipeline +
;; [method]gpu-compute-pass-encoder.set-pipeline
;; WIT: set-pipeline: func(pipeline: borrow<gpu-compute-pipeline>)
;; Guest borrows the pipeline; drops owns; run returns harness 1.
;; L2 described JNI forwards pass + pipeline reps (0 → stub in attach).
;; get-compute-pass / get-compute-pipeline are test constructors (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pipeline" (type $gpu-compute-pipeline (sub resource)))
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $borrow-pipe (borrow $gpu-compute-pipeline))
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $set-ty (func
      (param "self" $borrow-pass)
      (param "pipeline" $borrow-pipe)))
    (export "[method]gpu-compute-pass-encoder.set-pipeline" (func (type $set-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
    (type $own-pipe (own $gpu-compute-pipeline))
    (export "get-compute-pipeline" (func (result $own-pipe)))
  ))
  (alias export $webgpu "gpu-compute-pipeline" (type $gpu-compute-pipeline))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "get-compute-pipeline" (func $get-compute-pipeline))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.set-pipeline" (func $set-pipeline))

  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $gpipe_lower (canon lower (func $get-compute-pipeline)))
  (core func $sp_lower (canon lower (func $set-pipeline)))
  (core func $dpipe_lower (canon resource.drop $gpu-compute-pipeline))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "get-compute-pipeline" (func $get-compute-pipeline (result i32)))
    (import "" "set-pipeline" (func $set-pipeline (param i32 i32)))
    (import "" "drop-pipeline" (func $drop-pipeline (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local $pipe i32)
      (local.set $pass (call $get-compute-pass))
      (local.set $pipe (call $get-compute-pipeline))
      (call $set-pipeline (local.get $pass) (local.get $pipe))
      (call $drop-pipeline (local.get $pipe))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "get-compute-pipeline" (func $gpipe_lower))
      (export "set-pipeline" (func $sp_lower))
      (export "drop-pipeline" (func $dpipe_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
