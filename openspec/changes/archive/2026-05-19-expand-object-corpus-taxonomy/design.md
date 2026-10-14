## Context

The existing internal object corpus fixture is intentionally stable and covers `cargo-octet/src/*.rs`. The next validation gap is breadth across the repository's own non-CLI Rust modules without destabilizing the existing CI gate.

## Goals / Non-Goals

**Goals:** broaden internal corpus coverage, keep the existing core fixture intact, document selected source taxonomy, and assert deterministic replay over the expanded fixture.

**Non-Goals:** downstream corpora, HIR/MIR extraction, semantic-equivalence proof, correctness proof, or cross-machine determinism proof.

## Decisions

### 1. Add a separate expanded fixture

**Choice:** Add `expanded-internal-corpus-receipt.json` instead of replacing `internal-corpus-receipt.json`.

**Rationale:** Existing gate semantics stay stable while the expanded taxonomy can evolve independently.

**Alternative:** Replace the core fixture. Rejected because it would conflate broadening validation with the already-stabilized object corpus gate.

### 2. Use a taxonomy fixture as the source selection contract

**Choice:** Store the selected source paths and bucket names in a checked-in taxonomy JSON fixture.

**Rationale:** Tests can validate replay against an explicit inventory instead of hiding the selected modules inside Rust constants only.

### 3. Keep boundary assertions in tests

**Choice:** Expanded corpus tests assert `claims_correctness=false` and `claims_semantic_equivalence=false`.

**Rationale:** Broader coverage must not broaden semantic claims.

## Risks / Trade-offs

- **Fixture churn**: adding more modules increases fixture update frequency. Mitigation: separate expanded fixture from the core CI fixture.
- **Runtime cost**: broader corpus tests run more extraction work. Mitigation: keep the selected corpus internal and focused on stable modules.
