;; S6+: get-device + [method]gpu-device.create-query-set
;; WIT: create-query-set: func(descriptor) -> result<gpu-query-set, create-query-set-error>
;; Guest passes type=occlusion, count=1, label=none; drops own on ok; harness 1.
;; L2 described type/count (stub adapter→device when get-device rep=0).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $qt (enum "occlusion" "timestamp"))
    (export "gpu-query-type" (type $gpu-query-type (eq $qt)))
    (type $opt-str (option string))
    (type $desc-def (record
      (field "type" $gpu-query-type)
      (field "count" u32)
      (field "label" $opt-str)
    ))
    (export "gpu-query-set-descriptor" (type $desc (eq $desc-def)))
    (type $kind (variant (case "type-error")))
    (export "create-query-set-error-kind" (type $kind-eq (eq $kind)))
    (type $err (record (field "kind" $kind-eq) (field "message" string)))
    (export "create-query-set-error" (type $err-eq (eq $err)))
    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (export "gpu-device" (type $gpu-device (sub resource)))
    (type $borrow-device (borrow $gpu-device))
    (type $own-qs (own $gpu-query-set))
    (type $result-qs (result $own-qs (error $err-eq)))
    (type $create-ty (func
      (param "self" $borrow-device)
      (param "descriptor" $desc)
      (result $result-qs)))
    (export "[method]gpu-device.create-query-set" (func (type $create-ty)))
    (type $own-device (own $gpu-device))
    (export "get-device" (func (result $own-device)))
  ))
  (alias export $webgpu "gpu-query-set" (type $gpu-query-set))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-query-set" (func $create-qs))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cq_lower
    (canon lower (func $create-qs)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dq_lower (canon resource.drop $gpu-query-set))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-qs"
      (func $create-qs (param i32 i32 i32 i32 i32 i32 i32)))
    (import "" "drop-qs" (func $drop-qs (param i32)))
    (func (export "run") (result i32)
      (local $retptr i32)
      (local $tag i32)
      (local $handle i32)
      (local.set $retptr (i32.const 0))
      (call $create-qs
        (call $get-device)
        (i32.const 0)
        (i32.const 1)
        (i32.const 0)
        (i32.const 0)
        (i32.const 0)
        (local.get $retptr))
      (local.set $tag (i32.load (local.get $retptr)))
      (local.set $handle (i32.load offset=4 (local.get $retptr)))
      (if (i32.eqz (local.get $tag))
        (then (call $drop-qs (local.get $handle))))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "create-qs" (func $cq_lower))
      (export "drop-qs" (func $dq_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
