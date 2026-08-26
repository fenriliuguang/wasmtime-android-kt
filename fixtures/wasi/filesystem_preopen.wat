;; WASI 0.3 package smoke: wasi:filesystem directory preopen + open-at + read/write
;; Official packages: wasi:filesystem/types@0.3.0 + preopens@0.3.0.
;; get-directories → list<tuple<own<descriptor>, string>> (length 1, name ".").
;; open-at(path) -> result<descriptor, error-code> happy path "p3fs.txt".
;; write-via-stream(data, offset: filesize) / read-via-stream(offset) on the child.
;; Guest: get-directories → open-at("..") access → open-at("p3fs.txt") → write/read P3FS.
(component
  (import "wasi:filesystem/types@0.3.0" (instance $types
    (export "descriptor" (type $descriptor (sub resource)))
    (type $error-code-def (enum "unknown" "access"))
    (export "error-code" (type $error-code (eq $error-code-def)))
    (type $io-result (result (error $error-code)))
    (type $st (stream u8))
    (type $ft (future $io-result))
    (type $read-ret (tuple $st $ft))
    (type $borrow-desc (borrow $descriptor))
    (type $open-result (result (own $descriptor) (error $error-code)))
    (export "[method]descriptor.write-via-stream"
      (func (param "self" $borrow-desc) (param "data" $st) (param "offset" u64) (result $ft)))
    (export "[method]descriptor.read-via-stream"
      (func (param "self" $borrow-desc) (param "offset" u64) (result $read-ret)))
    (export "[method]descriptor.open-at"
      (func (param "self" $borrow-desc) (param "path" string) (result $open-result)))
  ))
  (alias export $types "descriptor" (type $descriptor))
  (alias export $types "error-code" (type $error-code))
  (alias export $types "[method]descriptor.write-via-stream" (func $write-via-stream))
  (alias export $types "[method]descriptor.read-via-stream" (func $read-via-stream))
  (alias export $types "[method]descriptor.open-at" (func $open-at))
  (import "wasi:filesystem/preopens@0.3.0" (instance $preopens
    (export "descriptor" (type (eq $descriptor)))
    (type $dir-tuple (tuple (own $descriptor) string))
    (export "get-directories" (func (result (list $dir-tuple))))
  ))
  (alias export $preopens "get-directories" (func $get-directories))
  (type $io-result (result (error $error-code)))
  (type $st (stream u8))
  (type $ft (future $io-result))

  (core module $libc
    (memory (export "mem") 1)
    (data (i32.const 16) "P3FS")
    (data (i32.const 96) "p3fs.txt")
    (data (i32.const 108) "..")
    (global $last (mut i32) (i32.const 256))
    (func (export "realloc")
      (param $oldptr i32) (param $oldlen i32) (param $align i32) (param $newlen i32)
      (result i32)
      (local $ret i32)
      (local.set $ret (global.get $last))
      (global.set $last
        (i32.and
          (i32.add (i32.add (local.get $ret) (local.get $newlen)) (i32.const 7))
          (i32.const -8)))
      (local.get $ret)
    )
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "mem" (memory 1))
    (import "" "stream.new" (func $stream.new (result i64)))
    (import "" "stream.write" (func $stream.write (param i32 i32 i32) (result i32)))
    (import "" "stream.read" (func $stream.read (param i32 i32 i32) (result i32)))
    (import "" "stream.drop-writable" (func $stream.drop-writable (param i32)))
    (import "" "future.read" (func $future.read (param i32 i32) (result i32)))
    (import "" "future.drop-readable" (func $future.drop-readable (param i32)))
    (import "" "get-directories" (func $get-directories (param i32)))
    (import "" "open-at" (func $open-at (param i32 i32 i32 i32)))
    (import "" "write-via-stream" (func $write-via-stream (param i32 i32 i64) (result i32)))
    (import "" "read-via-stream" (func $read-via-stream (param i32 i64 i32)))

    (func (export "run") (result i32)
      (local $dir i32)
      (local $desc i32)
      (local $list i32)
      (local $len i32)
      (local $pair i64)
      (local $r i32)
      (local $w i32)
      (local $fut i32)
      (local $s i32)
      (local $status i32)
      (local $n i32)

      ;; list<tuple<own, string>> at mem[80]: ptr, len. Index 0 is the sandbox dir.
      (call $get-directories (i32.const 80))
      (local.set $list (i32.load (i32.const 80)))
      (local.set $len (i32.load (i32.const 84)))
      (if (i32.eqz (local.get $len))
        (then unreachable))
      (local.set $dir (i32.load (local.get $list)))

      ;; open-at("..") → error-code.access (disc 1, payload 1).
      (call $open-at (local.get $dir) (i32.const 108) (i32.const 2) (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 1))
        (then unreachable))
      (if (i32.ne (i32.load8_u (i32.const 84)) (i32.const 1))
        (then unreachable))

      ;; result<descriptor, error-code> at mem[80]: u8 disc (0=ok), handle at 84.
      (call $open-at (local.get $dir) (i32.const 96) (i32.const 8) (i32.const 80))
      (if (i32.ne (i32.load8_u (i32.const 80)) (i32.const 0))
        (then unreachable))
      (local.set $desc (i32.load (i32.const 84)))

      (local.set $pair (call $stream.new))
      (local.set $r (i32.wrap_i64 (local.get $pair)))
      (local.set $w (i32.wrap_i64 (i64.shr_u (local.get $pair) (i64.const 32))))
      (local.set $fut (call $write-via-stream (local.get $desc) (local.get $r) (i64.const 0)))

      (local.set $status (call $stream.write (local.get $w) (i32.const 16) (i32.const 4)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $stream.drop-writable (local.get $w))

      (local.set $status (call $future.read (local.get $fut) (i32.const 0)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 0)) (i32.const 0))
        (then unreachable))

      ;; tuple at mem[32]: stream handle, future handle; offset 0
      (call $read-via-stream (local.get $desc) (i64.const 0) (i32.const 32))
      (local.set $s (i32.load (i32.const 32)))
      (local.set $fut (i32.load (i32.const 36)))

      (local.set $status (call $stream.read (local.get $s) (i32.const 48) (i32.const 16)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (local.set $n (i32.shr_u (local.get $status) (i32.const 4)))
      (if (i32.ne (i32.load (i32.const 48)) (i32.load (i32.const 16)))
        (then unreachable))

      (local.set $status (call $future.read (local.get $fut) (i32.const 64)))
      (if (i32.eq (local.get $status) (i32.const -1))
        (then unreachable))
      (call $future.drop-readable (local.get $fut))
      (if (i32.ne (i32.load8_u (i32.const 64)) (i32.const 0))
        (then unreachable))

      (local.get $n)
    )
  )

  (core func $stream.new (canon stream.new $st))
  (core func $stream.write (canon stream.write $st async (memory $libc "mem")))
  (core func $stream.read (canon stream.read $st async (memory $libc "mem")))
  (core func $stream.drop-writable (canon stream.drop-writable $st))
  (core func $future.read (canon future.read $ft async (memory $libc "mem")))
  (core func $future.drop-readable (canon future.drop-readable $ft))
  (core func $get_directories_lower
    (canon lower (func $get-directories)
      (memory $libc "mem")
      (realloc (func $libc "realloc"))))
  (core func $open_at_lower
    (canon lower (func $open-at) (memory $libc "mem")))
  (core func $write_lower (canon lower (func $write-via-stream) (memory $libc "mem")))
  (core func $read_lower (canon lower (func $read-via-stream) (memory $libc "mem")))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "mem" (memory $libc "mem"))
      (export "stream.new" (func $stream.new))
      (export "stream.write" (func $stream.write))
      (export "stream.read" (func $stream.read))
      (export "stream.drop-writable" (func $stream.drop-writable))
      (export "future.read" (func $future.read))
      (export "future.drop-readable" (func $future.drop-readable))
      (export "get-directories" (func $get_directories_lower))
      (export "open-at" (func $open_at_lower))
      (export "write-via-stream" (func $write_lower))
      (export "read-via-stream" (func $read_lower))
    ))
  ))

  (func (export "run") (result u32)
    (canon lift (core func $i "run")))
)
