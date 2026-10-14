# Change: Kani panic-freedom pilots

## Why

Kani is strongest as an incremental entry point: developers can first prove panic-freedom and boundary safety over symbolic inputs, then add contracts only where stronger properties matter. Octet should dogfood this path on representative critical Rust surfaces before broad guidance asks consumers to do the same.

## What Changes

- Define pilot selection criteria for pure functional-core modules, arithmetic and encoding boundaries, unsafe or FFI-adjacent wrappers, and state-transition code.
- Add a Kani harness convention for panic-freedom checks, reviewed assumptions, safety-invariant admission, and concrete counterexample playback.
- Require positive and negative pilot evidence so harnesses prove useful properties and also demonstrate fail-closed behavior for invalid assumptions or malformed inputs.
- Document that pilots are targeted evidence, not a replacement for Octet lints, property tests, Miri, or Verus proof rails.

## Impact

- **Files**: pilot guide, harness templates, fixture/pilot crates or modules, and evidence receipts.
- **Consumers**: future Octet users get copyable examples before adopting the rail.
- **Non-claims**: Pilot harnesses cover declared scopes only and do not establish whole-workspace correctness.
