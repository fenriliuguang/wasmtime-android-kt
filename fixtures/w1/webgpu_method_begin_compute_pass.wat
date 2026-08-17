;; S8: wasi:webgpu/webgpu@0.3.0-rc.2 get-encoder +
;; [method]gpu-command-encoder.begin-compute-pass
;; WIT: begin-compute-pass: func(descriptor: option<gpu-compute-pass-descriptor>)
;;      -> gpu-compute-pass-encoder
;; Guest passes descriptor=none; drops own pass; run returns harness 1.
;; get-encoder is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (type $borrow-qs (borrow $gpu-query-set))
    (type $opt-u32 (option u32))
    (type $ts-def (record
      (field "query-set" $borrow-qs)
      (field "beginning-of-pass-write-index" $opt-u32)
      (field "end-of-pass-write-index" $opt-u32)
    ))
    (export "gpu-compute-pass-timestamp-writes" (type (eq $ts-def)))
    (type $opt-str (option string))
    (type $opt-ts (option 4))
    (type $desc-def (record
      (field "timestamp-writes" $opt-ts)
      (field "label" $opt-str)
    ))
    (export "gpu-compute-pass-descriptor" (type (eq $desc-def)))
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-command-encoder))
    (type $opt-desc (option 8))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (type $begin-ty (func
      (param "self" $borrow-encoder)
      (param "descriptor" $opt-desc)
      (result $own-pass)))
    (export "[method]gpu-command-encoder.begin-compute-pass" (func (type $begin-ty)))
    (type $own-encoder (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.begin-compute-pass" (func $begin))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $ge_lower (canon lower (func $get-encoder)))
  (core func $b_lower
    (canon lower (func $begin)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dp_lower (canon resource.drop $gpu-compute-pass-encoder))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "begin"
      (func $begin
        (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-pass" (func $drop-pass (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $pass i32)
      (local.set $encoder (call $get-encoder))
      (local.set $pass
        (call $begin
          (local.get $encoder)
          (i32.const 0)
          (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
          (i32.const 0)))
      (call $drop-pass (local.get $pass))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "begin" (func $b_lower))
      (export "drop-pass" (func $dp_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
