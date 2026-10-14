# Design: Valence Project Extraction Phase 0

## Overview

Phase 0 is a planning and evidence-boundary slice. It prepares Valence for extraction by making the move/no-move boundary explicit before any code is relocated. The output is a reviewed Cairn package plus a checked-in inventory artifact, not a new standalone repository.

## Boundary model

### Candidate Valence-owned surface

The inventory treats these Octet-local surfaces as candidate Valence ownership because they implement or prove Rust object/evidence behavior:

- `cargo-octet/src/object.rs` — Function Object v1, object corpus, supplement receipt emit/check behavior, and HIR/THIR/MIR selected-fixture seams.
- `cargo-octet/src/evidence_index.rs` — generated current-head Valence evidence page and freshness checker.
- `cargo-octet/src/cli.rs` object/evidence-index command definitions — current CLI shell for the Valence-oriented behaviors.
- `cargo-octet/tests/fixtures/object-corpus/**` — object corpus, compatibility, caveat taxonomy, and freshness fixtures.
- `cargo-octet/tests/fixtures/{hir-spike,thir-spike,mir-spike}/**` — selected rustc fixture spike inputs.
- `docs/current-head-valence-evidence.md` — current human-readable evidence index.
- `cairn/specs/valence-evidence/spec.md` and archived Valence evidence Cairns — accepted evidence contracts and lifecycle history.
- Nix checks `valence-evidence-index`, `supplement-compatibility-gate`, `valence-evidence-release`, and `valence-object-evidence`.

### Octet-retained surface

Octet remains the lint/product shell until a later implementation slice proves otherwise:

- Root `octet` Dylint library and lint inventory.
- Tigerstyle/Slotcar compatibility surfaces and consumer rollout docs.
- `cargo-octet check`, fingerprint, transcript, migrate, roadmap, and non-Valence CLI behaviors unless a later package defines a stable Valence CLI split.
- Workspace/devshell wiring that builds the current combined repo.

### Shared handoff surface

Some files are intentionally shared until extraction work creates replacement locations:

- `flake.nix` gate composition and package build wiring.
- `Cargo.toml` workspace membership.
- `docs/content-addressed-rust-evidence-boundaries.md` and related architecture docs when they describe Octet/Valence evidence boundaries.
- Cairn lifecycle records that describe evidence contracts across both names.

## Required handoff gates before cutover

A future implementation/cutover package must preserve direct Nix entrypoints equivalent to:

- `nix build .#checks.x86_64-linux.valence-evidence-release --no-link -L`
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`
- `nix run .#cairn -- validate --root .`

It must also provide a compatibility or migration answer for the current explicit binary invocation used by the evidence checker:

```bash
cargo run -p cargo-octet --bin cargo-octet -- evidence-index check --page docs/current-head-valence-evidence.md
```

## Evidence artifact

`evidence/phase0-inventory.md` records the concrete paths and gates observed during this package. It is intentionally review-oriented and bounded: it identifies surfaces and preconditions, but does not assert that moving them is behavior-preserving.

## Risks and mitigations

- **Overclaiming readiness:** preserve non-claim lists in the spec and inventory.
- **CLI breakage:** require a compatibility answer before cutover.
- **Gate loss:** require named Nix checks in the handoff contract.
- **Repository split churn:** keep Phase 0 planning-only; code movement is a later Cairn package.
