;; S6+: get-render-bundle-encoder + [method]gpu-render-bundle-encoder.set-immediates
;; WIT: set-immediates: func(range-offset: u32, data: list<u8>,
;;      data-offset: option<u64>, data-size: option<u64>)
;; Guest passes range-offset=0, empty data, offset/size none; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-bundle-encoder" (type $gpu-render-bundle-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-render-bundle-encoder))
    (type $opt-u64 (option u64))
    (type $list-u8 (list u8))
    (type $imm-ty (func
      (param "self" $borrow-encoder)
      (param "range-offset" u32)
      (param "data" $list-u8)
      (param "data-offset" $opt-u64)
      (param "data-size" $opt-u64)))
    (export "[method]gpu-render-bundle-encoder.set-immediates" (func (type $imm-ty)))
    (type $own-encoder (own $gpu-render-bundle-encoder))
    (export "get-render-bundle-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "get-render-bundle-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-render-bundle-encoder.set-immediates" (func $set-imm))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $s_lower
    (canon lower (func $set-imm)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "set-imm"
      (func $set-imm (param i32 i32 i32 i32 i32 i64 i32 i64)))
    (func (export "run") (result i32)
      (call $set-imm
        (call $get-encoder)
        (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "set-imm" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
