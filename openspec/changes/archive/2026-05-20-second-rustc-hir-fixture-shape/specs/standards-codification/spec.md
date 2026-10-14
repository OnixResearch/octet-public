## MODIFIED Requirements

### Requirement: Rustc-backed HIR extractor spike [r[rustc-hir-extractor-spike]]

Octet MUST provide bounded, fixture-backed rustc HIR extractor spike evidence for selected Function Object v1 records while preserving the v1 object hash as the canonical review/provenance identity. Each supported HIR spike fixture shape MUST be a checked-in, explicitly selected fixture with exact rustc/toolchain identity, extractor identity, schema version, fixture context, observation hash, and explicit non-proof boundaries.

#### Scenario: Second HIR fixture shape attaches to Function Object v1

- GIVEN a second checked-in Rust fixture function with a branch or control-flow shape and an emitted `octet.function-object.v1` record
- WHEN Octet emits rustc-backed HIR spike evidence for that named fixture shape
- THEN the HIR evidence references the existing Function Object v1 `object_hash`, source path, and function name
- AND it records exact rustc/toolchain identity, extractor name/version, schema version, fixture-shape context, observation hash, and extraction caveats
- AND it does not replace the v1 object hash as review/provenance identity

#### Scenario: Second HIR fixture replay fails closed on shape mismatch

- GIVEN a saved rustc-backed HIR spike receipt for one supported fixture shape
- WHEN the receipt is checked against a different supported fixture shape, source path, function name, rustc/toolchain identity, extractor identity, schema version, or observation hash
- THEN replay fails with a deterministic mismatch report naming the compared field
- AND the report preserves no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain-stability boundary fields

#### Scenario: Multi-shape HIR spike support remains fixture bounded

- GIVEN rustc-backed HIR spike evidence exists for multiple checked-in fixture shapes
- WHEN contract metadata or operator documentation describes HIR supplement support
- THEN it names the support as selected-fixture-backed and toolchain-scoped rather than general HIR support
- AND THIR, MIR, arbitrary HIR inputs, and unimplemented fixture shapes remain planned or unsupported until their own deterministic positive and negative evidence exists
