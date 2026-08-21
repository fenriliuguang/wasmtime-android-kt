;; S1–S3 leftover: get-gpu + [method]gpu.request-adapter with options.feature-level="core".
;; Other option fields none; drops own adapter; harness 1.
;; get-gpu is a test constructor only (not product WIT).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (type $pp (enum "low-power" "high-performance"))
    (export "gpu-power-preference" (type (eq $pp)))
    (type $opt-str (option string))
    (type $opt-pp (option 1))
    (type $opt-bool (option bool))
    (type $opts-def (record
      (field "feature-level" $opt-str)
      (field "power-preference" $opt-pp)
      (field "force-fallback-adapter" $opt-bool)
      (field "xr-compatible" $opt-bool)
    ))
    (export "gpu-request-adapter-options" (type (eq $opts-def)))
    (export "gpu-adapter" (type $gpu-adapter (sub resource)))
    (export "gpu" (type $gpu (sub resource)))
    (type $borrow-gpu (borrow $gpu))
    (type $opt-opts (option 6))
    (type $own-adapter (own $gpu-adapter))
    (type $opt-adapter (option $own-adapter))
    (type $request-adapter-ty (func async
      (param "self" $borrow-gpu)
      (param "options" $opt-opts)
      (result $opt-adapter)))
    (export "[method]gpu.request-adapter" (func (type $request-adapter-ty)))
    (type $own-gpu (own $gpu))
    (type $get-gpu-ty (func (result $own-gpu)))
    (export "get-gpu" (func (type $get-gpu-ty)))
  ))
  (alias export $webgpu "gpu-adapter" (type $gpu-adapter))
  (alias export $webgpu "get-gpu" (func $get-gpu))
  (alias export $webgpu "[method]gpu.request-adapter" (func $request-adapter))

  (core module $builtins
    (memory (export "mem") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (i32.const 16)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gg_lower (canon lower (func $get-gpu)))
  (core func $ra_lower
    (canon lower (func $request-adapter)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $da_lower (canon resource.drop $gpu-adapter))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-gpu" (func $get-gpu (result i32)))
    (import "" "request-adapter"
      (func $request-adapter
        (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)))
    (import "" "drop-adapter" (func $drop-adapter (param i32)))
    (data (i32.const 32) "core")
    (func (export "run") (result i32)
      (local $gpu i32)
      (local $retptr i32)
      (local $tag i32)
      (local $handle i32)
      (local.set $retptr (i32.const 0))
      (local.set $gpu (call $get-gpu))
      (call $request-adapter
        (local.get $gpu)
        (i32.const 1)
        (i32.const 1) (i32.const 32) (i32.const 4)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0)
        (local.get $retptr))
      (local.set $tag (i32.load (local.get $retptr)))
      (local.set $handle (i32.load offset=4 (local.get $retptr)))
      (if (local.get $tag)
        (then (call $drop-adapter (local.get $handle))))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-gpu" (func $gg_lower))
      (export "request-adapter" (func $ra_lower))
      (export "drop-adapter" (func $da_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
