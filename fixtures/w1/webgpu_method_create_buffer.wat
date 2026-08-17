;; S4: wasi:webgpu/webgpu@0.3.0-rc.2 get-device + [method]gpu-device.create-buffer
;; WIT: create-buffer: func(descriptor: gpu-buffer-descriptor) -> gpu-buffer
;; Guest passes size=4, usage=COPY_DST|VERTEX, mapped/label=none;
;; drops own buffer; run returns harness 1 (not the method shape).
;; get-device is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $usage (flags "map-read" "map-write" "copy-src" "copy-dst" "index" "vertex" "uniform" "storage" "indirect" "query-resolve"))
    (export "gpu-buffer-usage" (type (eq $usage)))
    (type $opt-bool (option bool))
    (type $opt-str (option string))
    (type $desc-def (record
      (field "size" u64)
      (field "usage" 1)
      (field "mapped-at-creation" $opt-bool)
      (field "label" $opt-str)
    ))
    (export "gpu-buffer-descriptor" (type (eq $desc-def)))
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-buffer (own $gpu-buffer))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" 5)
      (result $own-buffer)))
    (export "[method]gpu-device.create-buffer" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (type $get-device-ty (func (result $own-device)))
    (export "get-device" (func (type $get-device-ty)))
  ))
  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-buffer" (func $create-buffer))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cb_lower
    (canon lower (func $create-buffer)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $db_lower (canon resource.drop $gpu-buffer))

  (core module $m
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-buffer"
      (func $create-buffer
        (param i32 i64 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local $buffer i32)
      (local.set $device (call $get-device))
      (local.set $buffer
        (call $create-buffer
          (local.get $device)
          (i64.const 4)
          (i32.const 40)
          (i32.const 0)
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
      (export "get-device" (func $gd_lower))
      (export "create-buffer" (func $cb_lower))
      (export "drop-buffer" (func $db_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
