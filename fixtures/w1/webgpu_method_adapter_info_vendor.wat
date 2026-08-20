;; S6+: get-adapter-info + [method]gpu-adapter-info.vendor
;; L2 described adapter handle → vendor (Cpu stub cpu-vendor); harness 1.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter-info" (type $gpu-adapter-info (sub resource)))
    (type $borrow-info (borrow $gpu-adapter-info))
    (type $ty (func (param "self" $borrow-info) (result string)))
    (export "[method]gpu-adapter-info.vendor" (func (type $ty)))
    (type $own-info (own $gpu-adapter-info))
    (export "get-adapter-info" (func (result $own-info)))
  ))
  (alias export $webgpu "get-adapter-info" (func $get-info))
  (alias export $webgpu "[method]gpu-adapter-info.vendor" (func $vendor))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gi_lower (canon lower (func $get-info)))
  (core func $v_lower
    (canon lower (func $vendor)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-info" (func $get-info (result i32)))
    (import "" "vendor" (func $vendor (param i32 i32)))
    (func (export "run") (result i32)
      (call $vendor (call $get-info) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-info" (func $gi_lower))
      (export "vendor" (func $v_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
