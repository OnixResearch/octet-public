## Context

Lean exposes a live proof state: target goal, assumptions, holes, and errors. Octet should adapt that as an evidence obligation state, not as an interactive theorem-proving session. The state is computed from existing Octet facts: policy IDs, proof rails, proof cards, proof receipts, provenance records, artifact identities, and proof-ref inputs.

## Decisions

### 1. Model obligations as pure data

**Choice:** Introduce a pure `ProofGoalState`-style core representation whose inputs are typed claims, bindings, receipts, proof cards, provenance records, and verifier outcomes.

**Rationale:** Goal-state explanations must be deterministic and testable without filesystem, process, verifier, or terminal state.

### 2. Render satisfied and missing evidence together

**Choice:** The output SHOULD include the checked claim, assumptions, required evidence, satisfied evidence, missing obligations, blockers, remediation commands, and explicit non-claims in stable order.

**Rationale:** Reviewers need to know both why a claim currently passes and what prevents promotion when it fails.

### 3. Treat missing evidence as holes, not soft success

**Choice:** Missing receipts, stale source/proof bindings, unsupported evidence tiers, absent proof cards, and unmatched proof refs MUST appear as explicit blockers and MUST NOT emit `formal_proof` provenance.

**Rationale:** Lean holes are valuable because they cannot be silently ignored. Octet should preserve that fail-closed property.

### 4. Keep Lean out of the required runtime path

**Choice:** This change MUST NOT require Lean, parse Lean syntax, or make interactive proof search part of starter `cargo octet check`.

**Rationale:** Octet remains a Rust-compatible checked path with opt-in formal proof rails.

## Risks / Trade-offs

- More detailed output can become noisy; use stable grouping and scoped selectors to keep it reviewable.
- Goal-state output must not imply proof completeness beyond the validated receipt and artifact boundaries.
- Existing explain output may need compatibility-preserving extensions rather than replacement.
