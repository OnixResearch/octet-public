## Why

Octet's purity model treats effects as first-class, but allocation is not in the effect vocabulary. `src/purity/effect_lints.rs` classifies `EffectClass` for environment, filesystem, IO, logging, network, process, randomness, runtime, spawn, and time, and `pure_contract_violation()` checks that list plus global state, interior mutability, mutating input, panic, and resource returns. A function marked `#[tigerstyle::pure]` that calls `Vec::new()`, `Box::new()`, `String::from()`, or `format!()` passes every existing check. The Jane Street OxCaml `[@zero_alloc]` article (theconsensus.dev, 2026-06-27) shows the value of inverting the allocation workflow: instead of profiling to find allocations after the fact, you declare a function zero-alloc and the compiler refuses the build when allocation creeps into the annotated body. Octet already has the annotation-driven workflow for effects; allocation is the obvious gap.

## What Changes

- Add a `zero_alloc_attr_contract` lint to the `purity-effect-lints` family, parallel to `pure_attr_contract`, gated on a recognized `#[tigerstyle::zero_alloc]` / `#[octet::zero_alloc]` / `#[zero_alloc]` marker.
- Promote allocation to a first-class effect: add an `EffectClass::Allocation` variant and a `PureViolation::Allocation` variant so the banned-effect checklist in `pure_attr_contract` and the new `zero_alloc_attr_contract` share one allocation classifier.
- Reuse the existing `supported_allocating_type` / zero-arg-constructor classifier from `src/performance/allocation_in_loop.rs` rather than duplicating the `alloc`/`std` def-path table. Direct source-shape checks only; no transitive whole-call-tree proof.
- Add a `[@zero_alloc assume]`-style auditable escape hatch that maps onto existing `unjustified_allow` machinery: any `allow` of `zero_alloc_attr_contract` MUST be explicit and justified, never silent.
- Register metadata, README, lint reference, default-level snapshot, fixture manifests, and downstream rollout notes.

## Impact

- **Files**: `src/purity/effect_lints.rs` (new `EffectClass::Allocation`, `PureViolation::Allocation`, `ZERO_ALLOC_ATTR_CONTRACT` lint + pass, marker recognition, escape-hatch reuse), `src/performance/allocation_in_loop.rs` (expose the existing classifier for reuse), `src/lib.rs` and `src/config.rs` (registration, inventory, default levels), `crates/octet-lint-metadata/src/lib.rs`, `ui_tests/zero_alloc_attr_contract.rs` + `.stderr`, `ui_tests/zero_alloc_attr_contract_allow.rs` + `.stderr`, `templates/dylint.toml`, `docs/lint-reference.md`, `README.md`, downstream rollout guidance.
- **Testing**: positive UI fixtures for `Vec::new()`, `Box::new()`, `String::from()`, `to_string()`, and `format!()` under the marker; negative UI fixtures for stack-locals, array init, primitive returns, and explicit `allow` with justification; classifier unit tests in `effect_lints.rs`; catalog freshness for README, lint reference, metadata, fixture manifests, default-level snapshots, and downstream rollout guidance.
