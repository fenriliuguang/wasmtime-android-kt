;; S6+: get-render-bundle-encoder + [method]gpu-render-bundle-encoder.finish
;; WIT: finish: func(descriptor: option<gpu-render-bundle-descriptor>)
;;      -> gpu-render-bundle
;; Guest passes descriptor=none; drops own bundle; run returns harness 1.
;; get-render-bundle-encoder is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $opt-str (option string))
    (type $desc-def (record (field "label" $opt-str)))
    (export "gpu-render-bundle-descriptor" (type (eq $desc-def)))
    (export "gpu-render-bundle" (type $gpu-render-bundle (sub resource)))
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $opt-desc (option 2))
    (type $own-bundle (own $gpu-render-bundle))
    (type $finish-ty (func
      (param "self" $borrow-encoder)
      (param "descriptor" $opt-desc)
      (result $own-bundle)))
    (export "[method]gpu-render-bundle-encoder.finish" (func (type $finish-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "gpu-render-bundle" (type $gpu-render-bundle))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.finish" (func $finish))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $f_lower
    (canon lower (func $finish)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $db_lower (canon resource.drop $gpu-render-bundle))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "finish" (func $finish (param i32 i32 i32 i32 i32) (result i32)))
    (import "" "drop-bundle" (func $drop-bundle (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $bundle i32)
      (local.set $encoder (call $get-encoder))
      (local.set $bundle
        (call $finish
          (local.get $encoder)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (call $drop-bundle (local.get $bundle))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "finish" (func $f_lower))
      (export "drop-bundle" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
