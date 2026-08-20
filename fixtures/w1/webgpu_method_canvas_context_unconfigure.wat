;; S6+: get-canvas-context + [method]gpu-canvas-context.unconfigure
;; Guest constructs context then unconfigures; run returns harness 1. L2 unused.
;; get-canvas-context is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-canvas-context" (type $gpu-canvas-context (sub resource)))
    (type $borrow-ctx (borrow $gpu-canvas-context))
    (type $unconfigure-ty (func (param "self" $borrow-ctx)))
    (export "[method]gpu-canvas-context.unconfigure" (func (type $unconfigure-ty)))
    (type $own-ctx (own $gpu-canvas-context))
    (export "get-canvas-context" (func (result $own-ctx)))
  ))
  (alias export $webgpu "get-canvas-context" (func $get-ctx))
  (alias export $webgpu "[method]gpu-canvas-context.unconfigure" (func $unconfigure))

  (core func $gc_lower (canon lower (func $get-ctx)))
  (core func $u_lower (canon lower (func $unconfigure)))

  (core module $m
    (import "" "get-canvas-context" (func $get-ctx (result i32)))
    (import "" "unconfigure" (func $unconfigure (param i32)))
    (func (export "run") (result i32)
      (call $unconfigure (call $get-ctx))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-canvas-context" (func $gc_lower))
      (export "unconfigure" (func $u_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
