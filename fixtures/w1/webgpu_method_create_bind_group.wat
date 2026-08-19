;; S6+: wasi:webgpu/webgpu@0.3.0-rc.2 get-device + get-bind-group-layout +
;; [method]gpu-device.create-bind-group
;; WIT: create-bind-group: func(descriptor: gpu-bind-group-descriptor) -> gpu-bind-group
;; Guest passes layout borrow + empty entries + label="l2"; drops own;
;; run returns harness 1.
;; get-device / get-bind-group-layout are test constructors only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "gpu-buffer" (type $gpu-buffer (sub resource)))
    (export "gpu-sampler" (type $gpu-sampler (sub resource)))
    (export "gpu-texture" (type $gpu-texture (sub resource)))
    (export "gpu-texture-view" (type $gpu-texture-view (sub resource)))
    (export "gpu-bind-group-layout" (type $gpu-bind-group-layout (sub resource)))
    (type $borrow-buffer (borrow $gpu-buffer))
    (type $opt-u64 (option u64))
    (type $buf-bind (record
      (field "buffer" $borrow-buffer)
      (field "offset" $opt-u64)
      (field "size" $opt-u64)
    ))
    (export "gpu-buffer-binding" (type (eq $buf-bind)))
    (type $borrow-sampler (borrow $gpu-sampler))
    (type $borrow-texture (borrow $gpu-texture))
    (type $borrow-view (borrow $gpu-texture-view))
    (type $resource-var (variant
      (case "gpu-buffer" $borrow-buffer)
      (case "gpu-buffer-binding" 8)
      (case "gpu-sampler" $borrow-sampler)
      (case "gpu-texture" $borrow-texture)
      (case "gpu-texture-view" $borrow-view)
    ))
    (export "gpu-binding-resource" (type (eq $resource-var)))
    (type $entry (record
      (field "binding" u32)
      (field "resource" 13)
    ))
    (export "gpu-bind-group-entry" (type (eq $entry)))
    (type $list-entry (list 15))
    (type $opt-str (option string))
    (type $borrow-bgl (borrow $gpu-bind-group-layout))
    (type $desc-def (record
      (field "layout" $borrow-bgl)
      (field "entries" $list-entry)
      (field "label" $opt-str)
    ))
    (export "gpu-bind-group-descriptor" (type (eq $desc-def)))
    (export "gpu-bind-group" (type $gpu-bind-group (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-bg (own $gpu-bind-group))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" 20)
      (result $own-bg)))
    (export "[method]gpu-device.create-bind-group" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
    (type $own-bgl (own $gpu-bind-group-layout))
    (export "get-bind-group-layout" (func (result $own-bgl)))
  ))
  (alias export $webgpu "gpu-bind-group" (type $gpu-bind-group))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "get-bind-group-layout" (func $get-bgl))
  (alias export $webgpu "[method]gpu-device.create-bind-group" (func $create-bg))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $gbgl_lower (canon lower (func $get-bgl)))
  (core func $cbg_lower
    (canon lower (func $create-bg)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dbg_lower (canon resource.drop $gpu-bind-group))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "get-bgl" (func $get-bgl (result i32)))
    (import "" "create-bg"
      (func $create-bg
        (param i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "drop-bg" (func $drop-bg (param i32)))
    (data (i32.const 32) "l2")
    (func (export "run") (result i32)
      (local $device i32)
      (local $layout i32)
      (local $bg i32)
      (local.set $device (call $get-device))
      (local.set $layout (call $get-bgl))
      (local.set $bg
        (call $create-bg
          (local.get $device)
          (local.get $layout)
          (i32.const 0)
          (i32.const 0)
          (i32.const 1)
          (i32.const 32)
          (i32.const 2)))
      (call $drop-bg (local.get $bg))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "get-bgl" (func $gbgl_lower))
      (export "create-bg" (func $cbg_lower))
      (export "drop-bg" (func $dbg_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
