(module
  (import "molten:hostcall" "assert" (func $assert (param i32 i32) (result i64)))
  (memory (export "memory") 1 2)
  (global $next (mut i32) (i32.const 1024))
  (func (export "molten_alloc") (param $len i32) (result i32)
    (local $ptr i32)
    global.get $next local.tee $ptr
    local.get $len i32.add global.set $next
    local.get $ptr)
  (func (export "molten_dealloc") (param $ptr i32) (param $len i32)
    local.get $ptr global.set $next)
  (func (export "molten_run") (param $ptr i32) (param $len i32) (result i64)
    local.get $ptr local.get $len call $assert))
