;; Static checker seed, not an implementation of Molten actor semantics.
(component
  (type $bytes (list u8))
  (import "preserves-payload" (type $payload (eq $bytes)))
  (type $outcome (result $payload (error string)))
  (type $request (func (param "request" $payload) (result $outcome)))
  (import "effect-request" (func $effect (type $request)))
  (export "invoke" (func $effect))
)
