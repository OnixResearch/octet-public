# Kani panic-freedom pilots

Use Kani pilots for small, reviewable Rust surfaces where symbolic input coverage is more valuable than another sampled test.

## Selection criteria

Prefer:

- deterministic functional-core code with no I/O, clocks, or environment reads;
- arithmetic, normalization, bounds, and encode/decode boundaries;
- unsafe or FFI-adjacent wrappers with explicit admitted-input preconditions;
- compact state transitions where every state can be represented symbolically;
- sparse boundary behavior that fuzzing may not reach.

Defer or model explicitly:

- async scheduling and thread interleavings;
- FFI behavior that is not stubbed with documented assumptions;
- harnesses whose `kani::assume` clauses make the input space empty.

## Harness convention

Each pilot names:

- property class from `docs/kani-property-checklist.md`;
- admitted input boundary and every `kani::assume` clause;
- Kani mode, solver, unwind or contract status;
- replay or concrete counterexample path when Kani reports one;
- artifact directory for the `kani-model-check` receipt.

## Positive and negative evidence

A pilot is not complete with only a happy path. Pair the passing harness with one negative fixture or replay note showing invalid input, malformed state, vacuous assumption, or failing assertion is rejected. Store the resulting summary through:

```bash
cargo octet evidence collect --rail kani-model-check --input target/kani-summary.json --artifact-dir artifacts/kani-model-check
```

## Graduation path

Panic-freedom is the entry point. Use passing unwind coverage or Kani contracts when the requirement claims unbounded coverage or functional correctness. Use Verus/Octet proof rails for richer proof obligations that require ghost state or reusable theorems.
