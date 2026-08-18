;; S6+: get-compute-pass + [method]gpu-compute-pass-encoder.set-immediates
;; WIT: set-immediates: func(range-offset: u32, data: list<u8>,
;;      data-offset: option<u64>, data-size: option<u64>)
;; Guest passes range-offset=0, empty data, offset/size none; harness 1.
;; L2 unused (lift-only).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $borrow-pass (borrow $gpu-compute-pass-encoder))
    (type $opt-u64 (option u64))
    (type $list-u8 (list u8))
    (type $imm-ty (func
      (param "self" $borrow-pass)
      (param "range-offset" u32)
      (param "data" $list-u8)
      (param "data-offset" $opt-u64)
      (param "data-size" $opt-u64)))
    (export "[method]gpu-compute-pass-encoder.set-immediates" (func (type $imm-ty)))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (export "get-compute-pass" (func (result $own-pass)))
  ))
  (alias export $webgpu "get-compute-pass" (func $get-compute-pass))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.set-immediates" (func $set-imm))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gp_lower (canon lower (func $get-compute-pass)))
  (core func $s_lower
    (canon lower (func $set-imm)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))

  (core module $m
    (import "" "get-compute-pass" (func $get-compute-pass (result i32)))
    (import "" "set-imm"
      (func $set-imm (param i32 i32 i32 i32 i32 i64 i32 i64)))
    (func (export "run") (result i32)
      (call $set-imm
        (call $get-compute-pass)
        (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i64.const 0)
        (i32.const 0) (i64.const 0))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-compute-pass" (func $gp_lower))
      (export "set-imm" (func $s_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
