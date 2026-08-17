;; S7: wasi:webgpu/webgpu@0.3.0-rc.2 get-encoder +
;; [method]gpu-command-encoder.finish
;; WIT: finish: func(descriptor: option<gpu-command-buffer-descriptor>)
;;      -> gpu-command-buffer
;; Guest passes descriptor=none; drops own buffer; run returns harness 1.
;; get-encoder is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $opt-str (option string))
    (type $desc-def (record (field "label" $opt-str)))
    (export "gpu-command-buffer-descriptor" (type (eq $desc-def)))
    (export "gpu-command-buffer" (type $gpu-command-buffer (sub resource)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-command-encoder))
    (type $opt-desc (option 2))
    (type $own-buffer (own $gpu-command-buffer))
    (type $finish-ty (func
      (param "self" $borrow-encoder)
      (param "descriptor" $opt-desc)
      (result $own-buffer)))
    (export "[method]gpu-command-encoder.finish" (func (type $finish-ty)))
    (type $own-encoder (own $gpu-command-encoder))
    (export "get-encoder" (func (result $own-encoder)))
  ))
  (alias export $webgpu "gpu-command-buffer" (type $gpu-command-buffer))
  (alias export $webgpu "get-encoder" (func $get-encoder))
  (alias export $webgpu "[method]gpu-command-encoder.finish" (func $finish))

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
  (core func $db_lower (canon resource.drop $gpu-command-buffer))

  (core module $m
    (import "" "get-encoder" (func $get-encoder (result i32)))
    (import "" "finish" (func $finish (param i32 i32 i32 i32 i32) (result i32)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $encoder i32)
      (local $buffer i32)
      (local.set $encoder (call $get-encoder))
      (local.set $buffer
        (call $finish
          (local.get $encoder)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (call $drop-buffer (local.get $buffer))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "get-encoder" (func $ge_lower))
      (export "finish" (func $f_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
