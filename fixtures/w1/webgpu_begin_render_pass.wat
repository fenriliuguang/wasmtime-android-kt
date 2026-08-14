;; W3: wasi:webgpu/webgpu@0.3.0-rc.2#command-encoder-begin-render-pass-clear
;; (transitional flat, sync). Not [method]gpu-command-encoder.begin-render-pass.
;; Guest: adapter → device → encoder → begin-render-pass-clear(encoder, stub view 23);
;; return pass u32. Stub view is not a surface texture (W4 present stays experimental).
(component
  (import "wasi:webgpu/webgpu@0.3.0-rc.2" (instance $webgpu
    (export "request-adapter" (func async (result u32)))
    (export "adapter-request-device" (func async (param "adapter" u32) (result u32)))
    (export "device-create-command-encoder" (func (param "device" u32) (result u32)))
    (export "command-encoder-begin-render-pass-clear"
      (func (param "encoder" u32) (param "view" u32) (result u32)))
  ))
  (alias export $webgpu "request-adapter" (func $request-adapter))
  (alias export $webgpu "adapter-request-device" (func $adapter-request-device))
  (alias export $webgpu "device-create-command-encoder" (func $device-create-command-encoder))
  (alias export $webgpu "command-encoder-begin-render-pass-clear" (func $begin-clear))

  (core module $m
    (import "" "request-adapter" (func $request-adapter (result i32)))
    (import "" "adapter-request-device" (func $adapter-request-device (param i32) (result i32)))
    (import "" "device-create-command-encoder" (func $device-create-command-encoder (param i32) (result i32)))
    (import "" "begin-clear" (func $begin-clear (param i32 i32) (result i32)))
    (func (export "run") (result i32)
      (local $adapter i32)
      (local $device i32)
      (local $encoder i32)
      (local.set $adapter (call $request-adapter))
      (local.set $device (call $adapter-request-device (local.get $adapter)))
      (local.set $encoder (call $device-create-command-encoder (local.get $device)))
      (call $begin-clear (local.get $encoder) (i32.const 23))
    )
  )
  (core func $ra_lower (canon lower (func $request-adapter)))
  (core func $rd_lower (canon lower (func $adapter-request-device)))
  (core func $enc_lower (canon lower (func $device-create-command-encoder)))
  (core func $beg_lower (canon lower (func $begin-clear)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "request-adapter" (func $ra_lower))
      (export "adapter-request-device" (func $rd_lower))
      (export "device-create-command-encoder" (func $enc_lower))
      (export "begin-clear" (func $beg_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
