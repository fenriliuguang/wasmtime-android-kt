;; S6+: get-texture + [method]gpu-texture.texture-binding-view-dimension
;; WIT: texture-binding-view-dimension: func() -> option<gpu-texture-view-dimension>.
;; Host returns none; harness 1. L2 described texture handle (stub 1×1 when get-texture rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $viewdim (enum "d1" "d2" "d2-array" "cube" "cube-array" "d3"))
    (export "gpu-texture-view-dimension" (type $gpu-texture-view-dimension (eq $viewdim)))
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (type $borrow-tex (borrow $gpu-texture))
    (type $opt-viewdim (option $gpu-texture-view-dimension))
    (type $ty-fn (func (param "self" $borrow-tex) (result $opt-viewdim)))
    (export "[method]gpu-texture.texture-binding-view-dimension" (func (type $ty-fn)))
    (type $own-tex (own $gpu-texture))
    (export "get-texture" (func (result $own-tex)))
  ))
  (alias export $webgpu "get-texture" (func $get-tex))
  (alias export $webgpu "[method]gpu-texture.texture-binding-view-dimension" (func $meth))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gt_lower (canon lower (func $get-tex)))
  (core func $m_lower
    (canon lower (func $meth)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-tex" (func $get-tex (result i32)))
    (import "" "meth" (func $meth (param i32 i32)))
    (func (export "run") (result i32)
      (local $tex i32)
      (local.set $tex (call $get-tex))
      (call $meth (local.get $tex) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-tex" (func $gt_lower))
      (export "meth" (func $m_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
