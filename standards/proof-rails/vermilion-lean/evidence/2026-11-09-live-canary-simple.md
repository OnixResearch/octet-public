# Live Vermilion provider canary

Date: 2026-11-09

## Pinned closure built

- Vermilion: `7c83bbca96940538eb282a67ca923893e5aa0e06`
- Lean: `4.33.0` (`leanprover/lean4:v4.33.0`, commit `d8b1897`)
- Mathlib cache: 8,690 files fetched with `lake exe cache get`
- Verus fork: `48600836424a79420589c56d86e720236a029811`
- Verus vstd build: 1,972 verified, 0 errors
- Vermilion Lean libraries: 3,382 build jobs completed
- Vermilion Rust workspace: built

## Canary command

```console
cd <vermilion-checkout>
env -u CARGO_TARGET_DIR CC=clang CXX=clang++ RUSTFLAGS='-C linker=clang' \
  ./examples/m1-pipeline/run.sh
```

The `CC`/`RUSTFLAGS` settings work around this host's broken Nix
`gcc-ld` wrapper; they do not change the provider pin or the generated
obligations.

## Observed result

```text
[vermilion] Verus front end: elaborating examples/m1-pipeline/simple.rs (--no-verify)
[vermilion] building the Vermilion tools
[vermilion] lowering SST to the textual IR
[vermilion] preparing the Lean environment (cached unless the library changed)
[vermilion] generating Lean obligations from the IR
[vermilion] Lean is judging the generated obligations of examples/m1-pipeline/simple.rs
[vrml-check] unit=.vermilion/check-simple-increment.lean elapsed_ms=5361
✓ all 3 obligations in examples/m1-pipeline/generated/simple/ (per-function units) verified by Lean — 3 newly checked.
proofs twins updated examples/m1-pipeline/proofs/simple/ (1 unit twins, 3 obligations, 0 sorry)
[vermilion] kernel-checking your proofs twinproofs twin identical to the generated module; skipping duplicate build
✓ proofs twin kernel-checked: examples/m1-pipeline/proofs/simple
```

## Scope and non-claims

This is a live provider-pipeline canary on Vermilion's `simple.rs`
(`increment(x: u32)`, `requires x < 100`, `ensures y == x + 1`). It shows
that the pinned Vermilion/Verus/Lean closure builds and that Lean
kernel-checks the generated obligations and proof twin.

It is **not** the Octet `increment_checked(value: u64) -> Option<u64>`
pilot contract. It does not check the u64 overflow/Option contract, the
Verus `nat`-to-`u64` refinement bridge, the Aeneas `u32`-to-`u64` bridge, or
any `cargo octet prove`/proof-card/Valence integration. No provider
soundness, refinement, whole-program, semantic-equivalence, or release
claim follows.
