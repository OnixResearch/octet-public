## Context

`checks.x86_64-linux.valence-object-evidence` currently composes selected object corpus replay, compatibility, caveat taxonomy, and fresh-build determinism rails. THIR fixture inventory coverage exists in Cargo tests, but the aggregate gate does not yet depend on it.

## Goals

- Add deterministic aggregate-gate coverage for THIR fixture inventory drift.
- Keep a focused named check so failures can be debugged without rebuilding the full aggregate.
- Preserve bounded evidence replay wording and non-proof boundaries.

## Non-Goals

- No new THIR selectors.
- No runtime broadening beyond selected checked-in THIR fixtures.
- No claims of general THIR extraction, correctness, semantic equivalence, pure-cache eligibility, or cross-toolchain stability.

## Decisions

### 1. Focused Nix check plus aggregate dependency

**Choice:** Add a `thir-supplement-inventory-gate` check that runs the integration-test filter covering THIR inventory roundtrip and aggregate boundary assertions, then make `valence-object-evidence` depend on that check.

**Rationale:** This follows the existing aggregate pattern: individual rails remain named and debuggable, while the aggregate gate composes them for CI/operator use.

### 2. Integration assertions own aggregate contract

**Choice:** Add integration coverage that asserts the THIR inventory lists exactly `checked-arithmetic` and `call-return`, excludes `branch-control-flow`, reports `pilot-supported` only for selected fixture entries, provides replay commands, and keeps contract metadata `unsupported-planned` outside explicit selected fixture runtime paths.

**Rationale:** The aggregate gate should fail on the operator-visible JSON/contract surfaces, not only on internal helper DTO state.

## Risks / Mitigations

- **Archive spec merge can drop existing scenarios.** Inspect the canonical spec diff after archive and rebuild the aggregate requirement from HEAD plus intended deltas if needed.
- **Aggregate builds can be slow/noisy.** Save focused rail evidence first, then aggregate evidence with exact command and exit status.

## Validation Plan

- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet thir_inventory -- --nocapture'`
- `nix build .#checks.x86_64-linux.thir-supplement-inventory-gate --no-link -L`
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`
- `openspec validate add-thir-inventory-aggregate-gate --strict --json`
- `openspec validate --all --strict --json`
- `git diff --check`
