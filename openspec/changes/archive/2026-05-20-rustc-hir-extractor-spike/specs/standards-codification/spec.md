## ADDED Requirements

### Requirement: Rustc-backed HIR extractor spike [r[rustc-hir-extractor-spike]]

Octet MUST provide a bounded, fixture-backed rustc HIR extractor spike that emits deterministic HIR observation evidence for selected Function Object v1 records while preserving the v1 object hash as the canonical review/provenance identity. The spike MUST record exact rustc/toolchain identity, extractor identity, schema version, fixture context, observation hash, and explicit non-proof boundaries.

#### Scenario: Fixture HIR extraction attaches to Function Object v1

- GIVEN a selected checked-in Rust fixture function with an emitted `octet.function-object.v1` record
- WHEN Octet emits rustc-backed HIR spike evidence for that fixture
- THEN the HIR evidence references the existing Function Object v1 `object_hash`, source path, and function name
- AND it records exact rustc/toolchain identity, extractor name/version, schema version, fixture context, observation hash, and extraction caveats
- AND it does not replace the v1 object hash as review/provenance identity

#### Scenario: HIR replay fails closed on context mismatch

- GIVEN a saved rustc-backed HIR spike receipt
- WHEN the receipt is checked against a different source path, function name, rustc/toolchain identity, extractor identity, schema version, or observation hash
- THEN replay fails with a deterministic mismatch report naming the compared field
- AND the report preserves no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain-stability boundary fields

#### Scenario: HIR spike support remains fixture bounded

- GIVEN the rustc-backed HIR spike only has deterministic evidence for selected fixtures
- WHEN contract metadata or operator documentation describes HIR supplement support
- THEN it names the spike as fixture-backed and toolchain-scoped rather than general HIR support
- AND THIR, MIR, and unimplemented non-fixture HIR families remain planned or unsupported until their own deterministic positive and negative evidence exists

#### Scenario: HIR evidence has no correctness overclaim

- GIVEN a rustc-backed HIR observation matches a saved receipt
- WHEN Octet reports the match to an operator or downstream consumer
- THEN it describes the result as bounded review/provenance supplement replay
- AND it does not claim behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, cross-machine determinism, or cross-toolchain determinism
