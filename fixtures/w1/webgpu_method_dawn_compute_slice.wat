;; Lane D: canonical [method] compute slice (no new WIT names).
;; get-device → create-buffer → create-command-encoder → queue →
;; begin-compute-pass (none) → end → finish (none) → submit.
;; Drops owns; run returns harness 1. get-device is a test constructor
;; (not product WIT); host attach caches one Dawn adapter/device when
;; guest device.rep is 0 so the chain shares a single GPU device.
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

    (type $enc-desc-def (record (field "label" $opt-str)))
    (export "gpu-command-encoder-descriptor" (type $enc-desc-export (eq $enc-desc-def)))
    (export "gpu-command-encoder" (type $gpu-command-encoder (sub resource)))
    (type $opt-enc-desc (option $enc-desc-export))
    (type $own-encoder (own $gpu-command-encoder))
    (type $create-enc-ty (func
      (param "self" $borrow-device)
      (param "descriptor" $opt-enc-desc)
      (result $own-encoder)))
    (export "[method]gpu-device.create-command-encoder" (func (type $create-enc-ty)))

    (export "gpu-queue" (type $gpu-queue (sub resource)))
    (type $own-queue (own $gpu-queue))
    (export "[method]gpu-device.queue"
      (func (param "self" $borrow-device) (result $own-queue)))

    (export "gpu-query-set" (type $gpu-query-set (sub resource)))
    (type $borrow-qs (borrow $gpu-query-set))
    (type $opt-u32 (option u32))
    (type $ts-def (record
      (field "query-set" $borrow-qs)
      (field "beginning-of-pass-write-index" $opt-u32)
      (field "end-of-pass-write-index" $opt-u32)
    ))
    (export "gpu-compute-pass-timestamp-writes" (type $ts-export (eq $ts-def)))
    (type $opt-ts (option $ts-export))
    (type $pass-desc-def (record
      (field "timestamp-writes" $opt-ts)
      (field "label" $opt-str)
    ))
    (export "gpu-compute-pass-descriptor" (type $pass-desc-export (eq $pass-desc-def)))
    (export "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder (sub resource)))
    (type $borrow-encoder (borrow $gpu-command-encoder))
    (type $opt-pass-desc (option $pass-desc-export))
    (type $own-pass (own $gpu-compute-pass-encoder))
    (type $begin-ty (func
      (param "self" $borrow-encoder)
      (param "descriptor" $opt-pass-desc)
      (result $own-pass)))
    (export "[method]gpu-command-encoder.begin-compute-pass" (func (type $begin-ty)))
    (export "[method]gpu-compute-pass-encoder.end"
      (func (param "self" (borrow $gpu-compute-pass-encoder))))

    (type $cb-desc-def (record (field "label" $opt-str)))
    (export "gpu-command-buffer-descriptor" (type $cb-desc-export (eq $cb-desc-def)))
    (export "gpu-command-buffer" (type $gpu-command-buffer (sub resource)))
    (type $opt-cb-desc (option $cb-desc-export))
    (type $own-cb (own $gpu-command-buffer))
    (type $finish-ty (func
      (param "self" $borrow-encoder)
      (param "descriptor" $opt-cb-desc)
      (result $own-cb)))
    (export "[method]gpu-command-encoder.finish" (func (type $finish-ty)))

    (type $borrow-queue (borrow $gpu-queue))
    (type $borrow-cb (borrow $gpu-command-buffer))
    (type $list-cb (list $borrow-cb))
    (type $submit-ty (func
      (param "self" $borrow-queue)
      (param "command-buffers" $list-cb)))
    (export "[method]gpu-queue.submit" (func (type $submit-ty)))
  ))

  (alias export $webgpu "gpu-buffer" (type $gpu-buffer))
  (alias export $webgpu "gpu-command-encoder" (type $gpu-command-encoder))
  (alias export $webgpu "gpu-compute-pass-encoder" (type $gpu-compute-pass-encoder))
  (alias export $webgpu "gpu-queue" (type $gpu-queue))
  (alias export $webgpu "gpu-command-buffer" (type $gpu-command-buffer))
  (alias export $webgpu "get-device" (func $get-device))
  (alias export $webgpu "[method]gpu-device.create-buffer" (func $create-buffer))
  (alias export $webgpu "[method]gpu-device.create-command-encoder" (func $create-encoder))
  (alias export $webgpu "[method]gpu-device.queue" (func $queue))
  (alias export $webgpu "[method]gpu-command-encoder.begin-compute-pass" (func $begin))
  (alias export $webgpu "[method]gpu-compute-pass-encoder.end" (func $end))
  (alias export $webgpu "[method]gpu-command-encoder.finish" (func $finish))
  (alias export $webgpu "[method]gpu-queue.submit" (func $submit))

  (core module $builtins
    (memory (export "mem") 1)
    (global $heap (mut i32) (i32.const 256))
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (local $ptr i32)
      (local.set $ptr (global.get $heap))
      (global.set $heap (i32.add (global.get $heap) (local.get 3)))
      (local.get $ptr)
    )
  )
  (core instance $builtins (instantiate $builtins))

  (core func $gd_lower (canon lower (func $get-device)))
  (core func $cb_lower
    (canon lower (func $create-buffer)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $ce_lower
    (canon lower (func $create-encoder)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $q_lower (canon lower (func $queue)))
  (core func $b_lower
    (canon lower (func $begin)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $e_lower (canon lower (func $end)))
  (core func $f_lower
    (canon lower (func $finish)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $s_lower
    (canon lower (func $submit)
      (memory $builtins "mem")
      (realloc (func $builtins "realloc"))))
  (core func $dbuf_lower (canon resource.drop $gpu-buffer))
  (core func $de_lower (canon resource.drop $gpu-command-encoder))
  (core func $dp_lower (canon resource.drop $gpu-compute-pass-encoder))
  (core func $dq_lower (canon resource.drop $gpu-queue))
  (core func $dcb_lower (canon resource.drop $gpu-command-buffer))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "get-device" (func $get-device (result i32)))
    (import "" "create-buffer"
      (func $create-buffer
        (param i32 i64 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "create-encoder"
      (func $create-encoder (param i32 i32 i32 i32 i32) (result i32)))
    (import "" "queue" (func $queue (param i32) (result i32)))
    (import "" "begin"
      (func $begin
        (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (result i32)))
    (import "" "end" (func $end (param i32)))
    (import "" "finish" (func $finish (param i32 i32 i32 i32 i32) (result i32)))
    (import "" "submit" (func $submit (param i32 i32 i32)))
    (import "" "drop-buffer" (func $drop-buffer (param i32)))
    (import "" "drop-encoder" (func $drop-encoder (param i32)))
    (import "" "drop-pass" (func $drop-pass (param i32)))
    (import "" "drop-queue" (func $drop-queue (param i32)))
    (import "" "drop-command-buffer" (func $drop-command-buffer (param i32)))
    (func (export "run") (result i32)
      (local $device i32)
      (local $buffer i32)
      (local $encoder i32)
      (local $queue i32)
      (local $pass i32)
      (local $cb i32)
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
      (local.set $encoder
        (call $create-encoder
          (local.get $device)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (local.set $queue (call $queue (local.get $device)))
      (local.set $pass
        (call $begin
          (local.get $encoder)
          (i32.const 0)
          (i32.const 0) (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)
          (i32.const 0) (i32.const 0)))
      (call $end (local.get $pass))
      (call $drop-pass (local.get $pass))
      (local.set $cb
        (call $finish
          (local.get $encoder)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)
          (i32.const 0)))
      (i32.store (i32.const 16) (local.get $cb))
      (call $submit (local.get $queue) (i32.const 16) (i32.const 1))
      (call $drop-command-buffer (local.get $cb))
      (call $drop-queue (local.get $queue))
      (call $drop-encoder (local.get $encoder))
      (call $drop-buffer (local.get $buffer))
      (i32.const 1)
    )
  )
  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $builtins "mem"))
      (export "get-device" (func $gd_lower))
      (export "create-buffer" (func $cb_lower))
      (export "create-encoder" (func $ce_lower))
      (export "queue" (func $q_lower))
      (export "begin" (func $b_lower))
      (export "end" (func $e_lower))
      (export "finish" (func $f_lower))
      (export "submit" (func $s_lower))
      (export "drop-buffer" (func $dbuf_lower))
      (export "drop-encoder" (func $de_lower))
      (export "drop-pass" (func $dp_lower))
      (export "drop-queue" (func $dq_lower))
      (export "drop-command-buffer" (func $dcb_lower))
    ))
  ))
  (func (export "run") async (result u32)
    (canon lift (core func $i "run"))
  )
)
