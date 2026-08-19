;; S6+: get-render-pipeline + [method]gpu-render-pipeline.get-bind-group-layout
;; WIT: get-bind-group-layout: func(index: u32) -> gpu-bind-group-layout. Drop own; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pipeline" (type $gpu-render-pipeline (sub resource)))
    (export "gpu-bind-group-layout" (type $gpu-bind-group-layout (sub resource)))
    (type $borrow-pipeline (borrow $gpu-render-pipeline))
    (type $own-layout (own $gpu-bind-group-layout))
    (type $ty (func (param "self" $borrow-pipeline) (param "index" u32) (result $own-layout)))
    (export "[method]gpu-render-pipeline.get-bind-group-layout" (func (type $ty)))
    (type $own-pipeline (own $gpu-render-pipeline))
    (export "get-render-pipeline" (func (result $own-pipeline)))
  ))
  (alias export $webgpu "gpu-bind-group-layout" (type $gpu-bind-group-layout))
  (alias export $webgpu "get-render-pipeline" (func $get-pipeline))
  (alias export $webgpu "[method]gpu-render-pipeline.get-bind-group-layout" (func $get-layout))

  (core func $grp_lower (canon lower (func $get-pipeline)))
  (core func $gl_lower (canon lower (func $get-layout)))
  (core func $dl_lower (canon resource.drop $gpu-bind-group-layout))

  (core module $m
    (import "" "get-pipeline" (func $get-pipeline (result i32)))
    (import "" "get-layout" (func $get-layout (param i32 i32) (result i32)))
    (import "" "drop-layout" (func $drop-layout (param i32)))
    (func (export "run") (result i32)
      (call $drop-layout (call $get-layout (call $get-pipeline) (i32.const 0)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pipeline" (func $grp_lower))
      (export "get-layout" (func $gl_lower))
      (export "drop-layout" (func $dl_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
