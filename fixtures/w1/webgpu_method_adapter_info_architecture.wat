;; S6+: get-adapter-info + [method]gpu-adapter-info.architecture
;; WIT: architecture: func() -> string. Host empty string; harness 1.
;; L2 unused (lift-only). get-adapter-info is a test constructor.
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-adapter-info" (type $gpu-adapter-info (sub resource)))
    (type $borrow-info (borrow $gpu-adapter-info))
    (type $ty (func (param "self" $borrow-info) (result string)))
    (export "[method]gpu-adapter-info.architecture" (func (type $ty)))
    (type $own-info (own $gpu-adapter-info))
    (export "get-adapter-info" (func (result $own-info)))
  ))
  (alias export $webgpu "get-adapter-info" (func $get-info))
  (alias export $webgpu "[method]gpu-adapter-info.architecture" (func $architecture))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gi_lower (canon lower (func $get-info)))
  (core func $fn_lower
    (canon lower (func $architecture)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-info" (func $get-info (result i32)))
    (import "" "architecture" (func $architecture (param i32 i32)))
    (func (export "run") (result i32)
      (call $architecture (call $get-info) (i32.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-info" (func $gi_lower))
      (export "architecture" (func $fn_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
