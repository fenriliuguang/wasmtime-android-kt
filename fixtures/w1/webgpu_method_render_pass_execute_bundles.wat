;; S6+: get-pass + get-render-bundle +
;; [method]gpu-render-pass-encoder.execute-bundles
;; WIT: execute-bundles: func(bundles: list<borrow<gpu-render-bundle>>)
;; Guest passes a one-element list; drops owns; run returns harness 1. L2 unused.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (export "gpu-render-bundle" (type $gpu-render-bundle (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $borrow-bundle (borrow $gpu-render-bundle))
    (type $list-bundle (list $borrow-bundle))
    (type $exec-ty (func
      (param "self" $borrow-pass)
      (param "bundles" $list-bundle)))
    (export "[method]gpu-render-pass-encoder.execute-bundles" (func (type $exec-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
    (type $own-bundle (own $gpu-render-bundle))
    (export "get-render-bundle" (func (result $own-bundle)))
  ))
  (alias export $webgpu "gpu-render-pass-encoder" (type $gpu-render-pass-encoder))
  (alias export $webgpu "gpu-render-bundle" (type $gpu-render-bundle))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "get-render-bundle" (func $get-render-bundle))
  (alias export $webgpu "[method]gpu-render-pass-encoder.execute-bundles" (func $execute))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $gb_lower (canon lower (func $get-render-bundle)))
  (core func $e_lower
    (canon lower (func $execute)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $db_lower (canon resource.drop $gpu-render-bundle))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "get-render-bundle" (func $get-render-bundle (result i32)))
    (import "" "execute" (func $execute (param i32 i32 i32)))
    (import "" "drop-bundle" (func $drop-bundle (param i32)))
    (func (export "run") (result i32)
      (local $pass i32)
      (local $bundle i32)
      (local.set $pass (call $get-pass))
      (local.set $bundle (call $get-render-bundle))
      (i32.store (i32.const 16) (local.get $bundle))
      (call $execute (local.get $pass) (i32.const 16) (i32.const 1))
      (call $drop-bundle (local.get $bundle))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-pass" (func $gp_lower))
      (export "get-render-bundle" (func $gb_lower))
      (export "execute" (func $e_lower))
      (export "drop-bundle" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
