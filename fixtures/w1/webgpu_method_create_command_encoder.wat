;; L2: wasi:webgpu/webgpu@0.3.0-rc.2 get-device +
;; [method]gpu-device.create-command-encoder
;; WIT: create-command-encoder: func(descriptor: option<gpu-command-encoder-descriptor>)
;;      -> gpu-command-encoder
;; Guest passes some(descriptor) label="l2"; drops own encoder; run returns harness 1.
;; get-device is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $opt-str (option string))
    (type $desc-def (record (field "label" $opt-str)))
    (export "gpu-command-encoder-descriptor" (type (eq $desc-def)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $opt-desc (option 2))
    (type $own-encoder (own $gpu-command-encoder))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" $opt-desc)
      (result $own-encoder)))
    (export "[method]gpu-device.create-command-encoder" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "gpu-command-encoder" (type $gpu-command-encoder))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-command-encoder" (func $create-encoder))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $ce_lower
    (canon lower (func $create-encoder)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $de_lower (canon resource.drop $gpu-command-encoder))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-encoder" (func $create-encoder (param i32 i32 i32 i32 i32) (result i32)))
    (import "" "drop-encoder" (func $drop-encoder (param i32)))
    (data (i32.const 32) "l2")
    (func (export "run") (result i32)
      (local $device i32)
      (local $encoder i32)
      (local.set $device (call $get-device))
      (local.set $encoder
        (call $create-encoder
          (local.get $device)
          (i32.const 1)
          (i32.const 1)
          (i32.const 32)
          (i32.const 2)))
      (call $drop-encoder (local.get $encoder))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "create-encoder" (func $ce_lower))
      (export "drop-encoder" (func $de_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
