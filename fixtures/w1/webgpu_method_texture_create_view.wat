;; W3+: get-texture + [method]gpu-texture.create-view (sync; host-fixed texture).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (export "get-texture" (func (result (own $gpu-texture))))
    (export "[method]gpu-texture.create-view"
      (func (param "self" (borrow $gpu-texture)) (result u32)))
  ))
  (alias export $webgpu "get-texture" (func $get-texture))
  (alias export $webgpu "[method]gpu-texture.create-view" (func $create-view))

  (core module $m
    (import "" "get-texture" (func $get-texture (result i32)))
    (import "" "create-view" (func $create-view (param i32) (result i32)))
    (func (export "run") (result i32)
      (local $texture i32)
      (local.set $texture (call $get-texture))
      (call $create-view (local.get $texture))
    )
  )
  (core func $gt_lower (canon lower (func $get-texture)))
  (core func $cv_lower (canon lower (func $create-view)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-texture" (func $gt_lower))
      (export "create-view" (func $cv_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
