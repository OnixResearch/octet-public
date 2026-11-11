# Vermilion u64 checked-increment pilot

This is the live Vermilion route for the bounded-increment pilot. It uses a
Verus-syntax source (`checked.rs`) with the same u64 checked contract as
`pilot/semantic-spec.txt`:

- precondition: `value < u64::MAX`
- postcondition: `result == checked_increment(value)`, i.e. `Some(value + 1)`
  under the precondition
- overflow behavior: checked; `None` at `u64::MAX`
- termination: single call, no loops

Command:

```console
env -u CARGO_TARGET_DIR CC=clang CXX=clang++ RUSTFLAGS='-C linker=clang' \
  ./scripts/run_example.sh examples/m1-pipeline checked.rs --lib M1Pipeline
```

Observed result: 2 obligations verified by Lean, 0 sorry, proof twin
kernel-checked. See `run.log`.

## Non-claims

The Verus-syntax source is a parallel specification, not the plain Rust
`pilot/src/lib.rs` executable. This run does not prove that the generated Lean
obligations refine the executable Rust function, does not check the Verus
`nat`-to-`u64` bridge, does not check the Aeneas `u32`-to-`u64` bridge, and does
not add `cargo octet prove`, proof-card, artifact-verification, or Valence
integration. It is measured provider evidence only.
