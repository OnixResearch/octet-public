## Context

Octet evidence comes from many producers: lints, policy checks, proof commands, external evidence importers, transcript validators, and artifact verifiers. A Lean-style kernel maps to Octet as a narrow deterministic checker for each promotion-sensitive claim, not as a single universal theorem kernel.

## Decisions

### 1. Distinguish producer and checker identity

**Choice:** Promotion-grade evidence SHOULD record the producer that emitted raw or derived material and the checker that accepted the claim when those roles are distinct.

**Rationale:** Reviewers need to know whether a claim was independently validated or merely asserted by its producer.

### 2. Keep checkers small and deterministic

**Choice:** Checker cores SHOULD accept normalized typed inputs and return pass/fail diagnostics, accepted facts, and non-claims without filesystem access, process execution, clocks, randomness, logging, or network state.

**Rationale:** Small deterministic checkers are easier to audit, unit test, and reuse across CLI surfaces.

### 3. Do not require independent checkers for every low-tier signal

**Choice:** The contract applies first to promotion-sensitive receipts and formal-proof evidence. Lower-tier lint, policy, transcript, external, or manual-review evidence MAY still identify a producer without implying independent proof.

**Rationale:** Not every signal has or needs a kernel-style checker, but promotion boundaries must be explicit.

### 4. Preserve existing wire compatibility through adapters

**Choice:** Existing receipt/provenance wire formats SHOULD be extended compatibly where possible, with adapters or schema versions when compatibility-sensitive fields change.

**Rationale:** Downstream artifact consumers should not break without a reviewed migration.

## Risks / Trade-offs

- Splitting producer/checker fields can create schema churn; limit initial scope to promotion-grade surfaces.
- A checker identity is only useful if the checker's accepted facts and non-claims are explicit.
- Avoid overclaiming that a small checker proves external tool soundness, solver soundness, or whole-program correctness.
