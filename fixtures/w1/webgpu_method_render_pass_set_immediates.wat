;; S6+: get-pass + [method]gpu-render-pass-encoder.set-immediates
;; WIT: set-immediates: func(range-offset: u32, data: list<u8>,
;;      data-offset: option<u64>, data-size: option<u64>)
;; Guest passes range-offset=0, empty data, offset/size none; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-render-pass-encoder" (type $gpu-render-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-render-pass-encoder))
    (type $opt-u64 (option u64))
    (type $list-u8 (list u8))
    (type $imm-ty (func
      (param "self" $borrow-pass)
      (param "range-offset" u32)
      (param "data" $list-u8)
      (param "data-offset" $opt-u64)
      (param "data-size" $opt-u64)))
    (export "[method]gpu-render-pass-encoder.set-immediates" (func (type $imm-ty)))
    (type $own-pass (own $gpu-render-pass-encoder))
    (export "get-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-pass" (func $get-pass))
  (alias export $webgpu "[method]gpu-render-pass-encoder.set-immediates" (func $set-imm))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gp_lower (canon lower (func $get-pass)))
  (core func $s_lower
    (canon lower (func $set-imm)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-pass" (func $get-pass (result i32)))
    (import "" "set-imm"
      (func $set-imm (param i32 i32 i32 i32 i32 i64 i32 i64)))
    (func (export "run") (result i32)
      (call $set-imm
        (call $get-pass)
        (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-pass" (func $gp_lower))
      (export "set-imm" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
