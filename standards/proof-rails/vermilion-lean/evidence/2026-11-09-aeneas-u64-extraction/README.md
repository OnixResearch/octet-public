# Aeneas extraction and theorem for the u64 checked-increment pilot

Date: 2026-11-09

This is the Aeneas/Charon route for the plain-Rust pilot
`increment_checked(value: u64) -> Option<u64>`. It uses the pinned provider
closure (`Aeneas bc2eb6f9`, `Charon cb50ff16`, Lean 4.31.0, Mathlib cache).

## Commands

```console
charon cargo --preset aeneas --dest-file llbc.llbc
aeneas -backend lean -emit-json -dest . -no-progress-bar llbc.llbc
LEAN_PATH=. lake env lean -o Llbc.olean Llbc.lean
LEAN_PATH=. lake env lean Theorem.lean
```

## Generated model

```lean
def increment_checked (value : Std.U64) : Result (Option Std.U64) := do
  if value < core.num.U64.MAX
  then let i ← value + 1#u64
       ok (some i)
  else ok none
```

## Checked theorem

`Theorem.lean` proves both contract cases over the generated model:

- `increment_checked_contract`: for `value < core.num.U64.MAX`, the model
  returns the checked `some (value + 1)` result.
- `increment_checked_at_max`: at `core.num.U64.MAX`, the model returns
  `ok none`.

Both check with the pinned Aeneas Lean library (exit 0, no diagnostics).

## Trust boundary and non-claims

The theorem is over the Aeneas-generated model, so it trusts the reviewed
Charon/Aeneas extraction and translation. It does not prove Charon/Aeneas
soundness, Lean kernel soundness, the Vermilion Verus-syntax source's
correspondence to this executable, whole-program correctness, semantic
equivalence, or release eligibility. The Vermilion executable bridge remains a
separate open obligation.
