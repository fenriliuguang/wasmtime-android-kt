;; S6+: get-render-bundle + [method]gpu-render-bundle.set-label
;; Guest passes empty label; run returns harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-bundle" (type $gpu-render-bundle (sub resource)))
    (type $borrow-res (borrow $gpu-render-bundle))
    (type $set-ty (func
      (param "self" $borrow-res)
      (param "label" string)))
    (export "[method]gpu-render-bundle.set-label" (func (type $set-ty)))
    (type $own-res (own $gpu-render-bundle))
    (export "get-render-bundle" (func (result $own-res)))
  ))
  (alias export $webgpu "get-render-bundle" (func $get-res))
  (alias export $webgpu "[method]gpu-render-bundle.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gr_lower (canon lower (func $get-res)))
  (core func $s_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-res" (func $get-res (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-res) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-res" (func $gr_lower))
      (export "set-label" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
