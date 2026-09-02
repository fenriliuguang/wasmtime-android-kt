;; GFX-PIN: wasi-gfx:surface@0.2.0 on-pointer-* / on-key-* streams.
;; Host registers all five pin imports. Guest constructs a surface, opens each
;; stream, drops it without reading (cancel), returns 1.
(component
  (import "wasi-gfx:surface/surface@0.2.0" (instance $surf
    (export "surface" (type $surface (sub resource)))
    (type $create-desc-def (record
      (field "height" (option u32))
      (field "width" (option u32))))
    (export "create-desc" (type $create-desc (eq $create-desc-def)))
    (type $pointer-event-def (record (field "x" f64) (field "y" f64)))
    (export "pointer-event" (type $pointer-event (eq $pointer-event-def)))
    (type $key-def (enum "backquote" "backslash" "bracket-left" "bracket-right" "comma" "digit0" "digit1" "digit2" "digit3" "digit4" "digit5" "digit6" "digit7" "digit8" "digit9" "equal" "intl-backslash" "intl-ro" "intl-yen" "key-a" "key-b" "key-c" "key-d" "key-e" "key-f" "key-g" "key-h" "key-i" "key-j" "key-k" "key-l" "key-m" "key-n" "key-o" "key-p" "key-q" "key-r" "key-s" "key-t" "key-u" "key-v" "key-w" "key-x" "key-y" "key-z" "minus" "period" "quote" "semicolon" "slash" "alt-left" "alt-right" "backspace" "caps-lock" "context-menu" "control-left" "control-right" "enter" "meta-left" "meta-right" "shift-left" "shift-right" "space" "tab" "convert" "kana-mode" "lang1" "lang2" "lang3" "lang4" "lang5" "non-convert" "delete" "end" "help" "home" "insert" "page-down" "page-up" "arrow-down" "arrow-left" "arrow-right" "arrow-up" "num-lock" "numpad0" "numpad1" "numpad2" "numpad3" "numpad4" "numpad5" "numpad6" "numpad7" "numpad8" "numpad9" "numpad-add" "numpad-backspace" "numpad-clear" "numpad-clear-entry" "numpad-comma" "numpad-decimal" "numpad-divide" "numpad-enter" "numpad-equal" "numpad-hash" "numpad-memory-add" "numpad-memory-clear" "numpad-memory-recall" "numpad-memory-store" "numpad-memory-subtract" "numpad-multiply" "numpad-paren-left" "numpad-paren-right" "numpad-star" "numpad-subtract" "escape" "f1" "f2" "f3" "f4" "f5" "f6" "f7" "f8" "f9" "f10" "f11" "f12" "fn" "fn-lock" "print-screen" "scroll-lock" "pause" "browser-back" "browser-favorites" "browser-forward" "browser-home" "browser-refresh" "browser-search" "browser-stop" "eject" "launch-app1" "launch-app2" "launch-mail" "media-play-pause" "media-select" "media-stop" "media-track-next" "media-track-previous" "power" "sleep" "audio-volume-down" "audio-volume-mute" "audio-volume-up" "wake-up" "hyper" "super" "turbo" "abort" "resume" "suspend" "again" "copy" "cut" "find" "open" "paste" "props" "select" "undo" "hiragana" "katakana"))
    (export "key" (type $key (eq $key-def)))
    (type $key-event-def (record
      (field "key" (option $key))
      (field "text" (option string))
      (field "alt-key" bool)
      (field "ctrl-key" bool)
      (field "meta-key" bool)
      (field "shift-key" bool)))
    (export "key-event" (type $key-event (eq $key-event-def)))
    (type $st-ptr (stream $pointer-event))
    (type $st-key (stream $key-event))
    (type $borrow-surf (borrow $surface))
    (export "[constructor]surface"
      (func (param "desc" $create-desc) (result (own $surface))))
    (export "[method]surface.on-pointer-up"
      (func (param "self" $borrow-surf) (result $st-ptr)))
    (export "[method]surface.on-pointer-down"
      (func (param "self" $borrow-surf) (result $st-ptr)))
    (export "[method]surface.on-pointer-move"
      (func (param "self" $borrow-surf) (result $st-ptr)))
    (export "[method]surface.on-key-up"
      (func (param "self" $borrow-surf) (result $st-key)))
    (export "[method]surface.on-key-down"
      (func (param "self" $borrow-surf) (result $st-key)))
  ))
  (alias export $surf "surface" (type $surface))
  (alias export $surf "pointer-event" (type $pointer-event))
  (alias export $surf "key-event" (type $key-event))
  (alias export $surf "[constructor]surface" (func $ctor))
  (alias export $surf "[method]surface.on-pointer-up" (func $ptr-up))
  (alias export $surf "[method]surface.on-pointer-down" (func $ptr-down))
  (alias export $surf "[method]surface.on-pointer-move" (func $ptr-move))
  (alias export $surf "[method]surface.on-key-up" (func $key-up))
  (alias export $surf "[method]surface.on-key-down" (func $key-down))
  (type $st-ptr (stream $pointer-event))
  (type $st-key (stream $key-event))

  (core module $libc
    (memory (export "mem") 1)
  )
  (core instance $libc (instantiate $libc))

  (core module $m
    (import "" "ctor" (func $ctor (param i32 i32 i32 i32) (result i32)))
    (import "" "ptr-up" (func $ptr-up (param i32) (result i32)))
    (import "" "ptr-down" (func $ptr-down (param i32) (result i32)))
    (import "" "ptr-move" (func $ptr-move (param i32) (result i32)))
    (import "" "key-up" (func $key-up (param i32) (result i32)))
    (import "" "key-down" (func $key-down (param i32) (result i32)))
    (import "" "ptr.drop" (func $ptr.drop (param i32)))
    (import "" "key.drop" (func $key.drop (param i32)))

    (func (export "run") (result i32)
      (local $surf i32)
      (local.set $surf (call $ctor
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
      (call $ptr.drop (call $ptr-up (local.get $surf)))
      (call $ptr.drop (call $ptr-down (local.get $surf)))
      (call $ptr.drop (call $ptr-move (local.get $surf)))
      (call $key.drop (call $key-up (local.get $surf)))
      (call $key.drop (call $key-down (local.get $surf)))
      (i32.const 1)
    )
  )

  (core func $ctor_lower (canon lower (func $ctor)))
  (core func $ptr_up_lower (canon lower (func $ptr-up)))
  (core func $ptr_down_lower (canon lower (func $ptr-down)))
  (core func $ptr_move_lower (canon lower (func $ptr-move)))
  (core func $key_up_lower (canon lower (func $key-up)))
  (core func $key_down_lower (canon lower (func $key-down)))
  (core func $ptr.drop (canon stream.drop-readable $st-ptr))
  (core func $key.drop (canon stream.drop-readable $st-key))

  (core instance $i (instantiate $m
    (with "" (instance
      (export "ctor" (func $ctor_lower))
      (export "ptr-up" (func $ptr_up_lower))
      (export "ptr-down" (func $ptr_down_lower))
      (export "ptr-move" (func $ptr_move_lower))
      (export "key-up" (func $key_up_lower))
      (export "key-down" (func $key_down_lower))
      (export "ptr.drop" (func $ptr.drop))
      (export "key.drop" (func $key.drop))
    ))
  ))

  (func (export "run") async (result u32)
    (canon lift (core func $i "run")))
)
