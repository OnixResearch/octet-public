## ADDED Requirements

### Requirement: Function Object Corpus Fresh-Target Determinism [r[function-object-corpus-fresh-target-determinism]]

Octet MUST verify that object corpus receipt generation remains stable across repeated local runs that use distinct fresh target directories and a scrubbed deterministic environment, while preserving the receipt boundary that hashes identify review/provenance evidence rather than correctness or semantic equivalence.

#### Scenario: Fresh target byte stability

- GIVEN the internal object corpus source selection
- WHEN Octet generates object corpus receipts multiple times with distinct fresh `CARGO_TARGET_DIR` values
- THEN the generated receipt bytes match exactly across runs

#### Scenario: Fresh target compared field stability

- GIVEN the generated fresh-target receipts
- WHEN the test compares the replay-significant receipt fields
- THEN object count, source paths, object-set hash, caveat counts, pure-cache blocker count, and boundary fields match across runs

#### Scenario: Fresh target bounded non-claims

- GIVEN the generated fresh-target receipts
- WHEN the boundary fields are inspected
- THEN `claims_correctness` and `claims_semantic_equivalence` remain false and the evidence is described as bounded local repeatability rather than cross-machine or semantic proof
