(component
  (type (;0;)
    (instance
      (type (;0;) (record (field "a" u32)))
      (export (;1;) "my-struct" (type (eq 0)))
      (type (;2;) (func (param "a" 0) (result string)))
      (export (;0;) "my-function" (func (type 2)))
    )
  )
  (export (;1;) "foo" (type 0))
)