## Why

Octet's closed-interface model needs a release-time check for public Rust API compatibility. Existing Octet architecture checks can budget or shape public API surface, but they do not compare a release candidate against an earlier API baseline. `cargo-semver-checks` is the strongest existing Rust-native tool for this job and should be integrated rather than reimplemented.

A first-class API-stability rail lets release operators record which baseline was checked, which crates were in scope, what feature/target set was used, and which SemVer findings block promotion.

## What Changes

- Add an API-stability rail backed by `cargo-semver-checks` for library crates.
- Require the operator or workspace metadata to name the baseline source, checked packages, feature set, target triple, and release intent.
- Emit deterministic API-stability receipts and provenance with `policy_check` evidence.
- Document that API-stability checks complement, but do not replace, human API review.

## Non-Goals

- Do not claim every source-compatible or behavior-compatible change is detected.
- Do not run API-stability checks in starter `cargo octet check` by default.
- Do not support non-Rust public API surfaces in this slice.
- Do not replace crate versioning decisions with automated version bumps.

## Impact

- **Files**: `cargo-octet/src/`, workspace metadata docs, release docs, tests, Nix wrapper inputs, and Cairn specs.
- **Behavior**: new release/API-stability rail that can fail closed on SemVer findings or invalid baselines.
- **Testing**: positive compatible fixture, negative removed public item fixture, missing baseline test, feature/target receipt tests, and Cairn gates.
