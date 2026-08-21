;; S6+: get-canvas-context + [method]gpu-canvas-context.get-current-texture
;; WIT: (borrow) -> own<gpu-texture>. Guest drops own; run returns harness 1. L2 unused.
;; get-canvas-context is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-canvas-context" (type $gpu-canvas-context (sub resource)))
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (type $borrow-ctx (borrow $gpu-canvas-context))
    (type $own-texture (own $gpu-texture))
    (type $get-ty (func (param "self" $borrow-ctx) (result $own-texture)))
    (export "[method]gpu-canvas-context.get-current-texture" (func (type $get-ty)))
    (type $own-ctx (own $gpu-canvas-context))
    (export "get-canvas-context" (func (result $own-ctx)))
  ))
  (alias export $webgpu "gpu-texture" (type $gpu-texture))
  (alias export $webgpu "get-canvas-context" (func $get-ctx))
  (alias export $webgpu "[method]gpu-canvas-context.get-current-texture" (func $get-tex))

  (core func $gc_lower (canon lower (func $get-ctx)))
  (core func $gt_lower (canon lower (func $get-tex)))
  (core func $dt_lower (canon resource.drop $gpu-texture))

  (core module $m
    (import "" "get-canvas-context" (func $get-ctx (result i32)))
    (import "" "get-current-texture" (func $get-tex (param i32) (result i32)))
    (import "" "drop-texture" (func $drop-texture (param i32)))
    (func (export "run") (result i32)
      (call $drop-texture (call $get-tex (call $get-ctx)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-canvas-context" (func $gc_lower))
      (export "get-current-texture" (func $gt_lower))
      (export "drop-texture" (func $dt_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
