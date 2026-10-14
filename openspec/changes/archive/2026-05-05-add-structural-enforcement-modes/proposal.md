## Why

Tigerstyle can already observe violations with lints and hard-gate them in CI, but several Tiger Style rules are stronger when Rust makes violations impossible by construction. We need a first-class OpenSpec for policy modes that can require crate boundaries, capability APIs, bounded primitives, type-state, or formal proofs, while still allowing ordinary lint-only rollout where structural enforcement is not yet practical.

## What Changes

- Add an enforcement-mode model for policies: lint, gate, structural, verified, and human/oracle.
- Define the Nickel-typed policy contracts for enforcement modes, mechanisms, evidence, suppressions, and seed manifests before Rust checkers consume exported machine data.
- Require `cargo-tigerstyle`/standards tooling to validate structural claims such as no-std core crates, shell-only effects, approved capabilities, bounded primitives, type-state protocols, and Verus proof rails.
- Define vendored seed-corpus rules for importing examples or reusable code from `../vericoder-mvp/`, `../verified-logic/`, and `../ucan/` only after provenance, hashes, licensing, and local verification are recorded.

## Non-Goals

- This change does not implement the enforcement engine or vendor code immediately.
- This change does not make every Tiger Style policy structural; weak static signals may remain lint-only, advisory, or human-review.
- This change does not bless sibling repository outputs as proof without local Tigerstyle verification.

## Capabilities

### New Capabilities

- `structural-enforcement-modes`: policies can require Rust-level construction mechanisms rather than only diagnostics.
- `reference-seed-corpus`: selected sibling repo code/examples can be vendored as reviewed seeds with deterministic provenance and re-verification.

### Modified Capabilities

- `tigerstyle-policy-registry`: policy entries gain current/target enforcement mode, typed Nickel contracts, cross-field validation, and accepted mechanism metadata.
- `tigerstyle-standards`: standards checks can fail when a project claims structural or verified enforcement but the Nickel config, crate graph, API shape, or proof evidence is missing.

## Impact

- **Files / artifacts**: `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/{proposal.md,design.md,tasks.md,specs/standards-codification/spec.md}`. Future implementation may touch `standards/policies/`, `src/`, `cargo-tigerstyle/`, docs, templates, Nix checks, and a controlled `vendor/` or fixture path.
- **Ordering / dependencies**: land this spec before implementation. Any vendoring from `../vericoder-mvp/`, `../verified-logic/`, or `../ucan/` must be preceded by a manifest containing source commit, file list, hashes, license review, intended role, and local verification command.
- **Verification / evidence**: planned evidence includes policy-schema validation, positive/negative structural fixtures, sibling-seed manifest validation, no-std/core-shell fixture checks, capability/type-state fixture checks, and Verus/proof-rail smoke checks where verified mode is selected.
