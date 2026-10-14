## Why

The `deterministic-core` profile currently admits configured direct Rust source and HIR shapes, but a selected function can still call another local function or dependency API whose effects were not checked at the caller. The Valence shared-identity pilot deliberately recorded that boundary rather than promoting direct checks into a transitive claim. Octet now needs a separate, fail-closed admission layer that computes a bounded effect closure over selected source-token functions and carries exact evidence for every admitted external edge.

## What Changes

- Add an opt-in `cargo octet admit --profile deterministic-core --closure` command that runs the existing strict-profile check first and attempts closure admission only after that check succeeds.
- Add a typed Nickel-authored effect taxonomy, profile allowance, role-edge catalog, and workspace-policy contract with checked-in generated runtime inputs.
- Add a pure deterministic Rust core that infers conservative direct effects and syntactic call edges from selected Rust source, resolves selected local functions, applies exact call-site dependency summaries, computes fixed-point transitive closure, and fails closed on unknown effects or unadmitted edges.
- Bind function addresses, selected source identities, profile/config identities, Cargo dependency identity, target/toolchain identity, workspace policy identity, closure nodes, and diagnostics into BLAKE3-addressed receipts.
- Add `cargo octet admit verify --artifact-dir PATH` for deterministic evidence-integrity verification without rerunning Cargo or claiming behavioral correctness.
- Dogfood the command against the existing Valence shared-identity pilot and retain both the initial fail-closed report and the final bounded admission evidence.

## Impact

- **CLI**: New `admit` and `admit verify` surfaces; existing `cargo octet check` behavior remains unchanged.
- **Policy**: New Nickel-authored effect-closure catalog and workspace-policy contract; runtime consumes generated JSON only.
- **Rust**: New source-analysis/closure core and thin filesystem, Cargo metadata, orchestration, and artifact shells in `cargo-octet`.
- **Evidence**: New versioned effect-closure receipt, status, summary, provenance, and integrity-verification surfaces.
- **Downstream**: Valence gains exact role and dependency-call summaries for its two selected production identity files; broader `valence-core` remains unadmitted.
- **Testing**: Positive and negative core, CLI, integration, Nickel fixture/freshness, replay, tamper, and downstream tests.

The change does not claim MIR call-graph completeness, trait-dispatch completeness, macro semantic expansion, dependency implementation verification, runtime call ordering, compiler correctness, whole-program correctness, deterministic execution, or release eligibility.
