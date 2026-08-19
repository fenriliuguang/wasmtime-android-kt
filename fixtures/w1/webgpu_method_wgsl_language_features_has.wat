;; S6+: get-gpu + [method]gpu.wgsl-language-features + [method]wgsl-language-features.has
;; WIT: has(value: string) -> bool. Host returns false; harness 1. L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu" (type $gpu (sub resource)))
    (export "wgsl-language-features" (type $wgsl-language-features (sub resource)))
    (type $borrow-gpu (borrow $gpu))
    (type $own-feat (own $wgsl-language-features))
    (type $feat-ty (func (param "self" $borrow-gpu) (result $own-feat)))
    (export "[method]gpu.wgsl-language-features" (func (type $feat-ty)))
    (type $own-gpu (own $gpu))
    (export "get-gpu" (func (result $own-gpu)))
    (type $borrow-feat (borrow $wgsl-language-features))
    (type $has-ty (func (param "self" $borrow-feat) (param "value" string) (result bool)))
    (export "[method]wgsl-language-features.has" (func (type $has-ty)))
  ))
  (alias export $webgpu "get-gpu" (func $get-gpu))
  (alias export $webgpu "[method]gpu.wgsl-language-features" (func $features))
  (alias export $webgpu "[method]wgsl-language-features.has" (func $has))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gg_lower (canon lower (func $get-gpu)))
  (core func $f_lower (canon lower (func $features)))
  (core func $h_lower
    (canon lower (func $has)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-gpu" (func $get-gpu (result i32)))
    (import "" "features" (func $features (param i32) (result i32)))
    (import "" "has" (func $has (param i32 i32 i32) (result i32)))
    (func (export "run") (result i32)
      (drop (call $has (call $features (call $get-gpu)) (i32.const 0) (i32.const 0)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-gpu" (func $gg_lower))
      (export "features" (func $f_lower))
      (export "has" (func $h_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
