;; L2: get-gpu + [method]gpu.wgsl-language-features
;; WIT: wgsl-language-features: func() -> own<wgsl-language-features>. Drop own; harness 1.
;; L2 unused (lift-only). get-gpu is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu" (type $gpu (sub resource)))
    (export "wgsl-language-features" (type $wgsl-language-features (sub resource)))
    (type $borrow-gpu (borrow $gpu))
    (type $own-feat (own $wgsl-language-features))
    (type $ty (func (param "self" $borrow-gpu) (result $own-feat)))
    (export "[method]gpu.wgsl-language-features" (func (type $ty)))
    (type $own-gpu (own $gpu))
    (export "get-gpu" (func (result $own-gpu)))
  ))
  (alias export $webgpu "wgsl-language-features" (type $wgsl-language-features))
  (alias export $webgpu "get-gpu" (func $get-gpu))
  (alias export $webgpu "[method]gpu.wgsl-language-features" (func $features))

  (core func $gg_lower (canon lower (func $get-gpu)))
  (core func $f_lower (canon lower (func $features)))
  (core func $df_lower (canon resource.drop $wgsl-language-features))

  (core module $m
    (import "" "get-gpu" (func $get-gpu (result i32)))
    (import "" "features" (func $features (param i32) (result i32)))
    (import "" "drop-feat" (func $drop-feat (param i32)))
    (func (export "run") (result i32)
      (call $drop-feat (call $features (call $get-gpu)))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-gpu" (func $gg_lower))
      (export "features" (func $f_lower))
      (export "drop-feat" (func $df_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
