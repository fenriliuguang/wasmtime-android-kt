;; S6+: get-texture-view + [method]gpu-texture-view.set-label
;; WIT: set-label: func(label: string). Guest passes empty label; harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-texture-view" (type $gpu-texture-view (sub resource)))
    (type $borrow-view (borrow $gpu-texture-view))
    (type $ty (func (param "self" $borrow-view) (param "label" string)))
    (export "[method]gpu-texture-view.set-label" (func (type $ty)))
    (type $own-view (own $gpu-texture-view))
    (export "get-texture-view" (func (result $own-view)))
  ))
  (alias export $webgpu "get-texture-view" (func $get-view))
  (alias export $webgpu "[method]gpu-texture-view.set-label" (func $set-label))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gv_lower (canon lower (func $get-view)))
  (core func $sl_lower
    (canon lower (func $set-label)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-view" (func $get-view (result i32)))
    (import "" "set-label" (func $set-label (param i32 i32 i32)))
    (func (export "run") (result i32)
      (call $set-label (call $get-view) (i32.const 0) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-view" (func $gv_lower))
      (export "set-label" (func $sl_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
