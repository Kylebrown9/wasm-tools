(component
  (type (;0;)
    (instance
      (type (;0;) u8)
      (export (;1;) "a" (type (eq 0)))
      (alias outer 0 0 (type (;2;)))
      (export (;3;) "b" (type (eq 2)))
      (type (;4;) (func (param "a" 0) (result 2)))
      (export (;0;) "f" (func (type 4)))
    )
  )
  (import "foo" (instance (;0;) (type 0)))
  (core module (;0;)
    (type (;0;) (func (param i32) (result i32)))
    (type (;1;) (func (param i32 i32 i32 i32) (result i32)))
    (import "foo" "f" (func (;0;) (type 0)))
    (func (;1;) (type 0) (param i32) (result i32)
      unreachable
    )
    (func (;2;) (type 1) (param i32 i32 i32 i32) (result i32)
      unreachable
    )
    (memory (;0;) 0)
    (export "foo#f" (func 1))
    (export "memory" (memory 0))
    (export "cabi_realloc" (func 2))
  )
  (alias export 0 "f" (func (;0;)))
  (core func (;0;) (canon lower (func 0)))
  (core instance (;0;)
    (export "f" (func 0))
  )
  (core instance (;1;) (instantiate 0
      (with "foo" (instance 0))
    )
  )
  (alias core export 1 "memory" (core memory (;0;)))
  (alias core export 1 "cabi_realloc" (core func (;1;)))
  (alias core export 1 "foo#f" (core func (;2;)))
  (type (;1;) u8)
  (alias outer 0 1 (type (;2;)))
  (type (;3;) (func (param "a" 1) (result 2)))
  (func (;1;) (type 3) (canon lift (core func 2)))
  (instance (;1;)
    (export "f" (func 1))
    (export "a" (type 1))
    (export "b" (type 2))
  )
  (export (;2;) "foo" (instance 1))
)