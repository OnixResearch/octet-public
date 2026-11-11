## Context

`cargo octet object corpus receipt` emits deterministic JSON from Rust DTOs and `cargo octet object corpus check` validates replay/drift. The recently accepted file-format contract says JSON is the current interchange and Rust DTOs/version constants are authoritative, with future Nickel/contract metadata allowed as a policy layer. This slice keeps the source of truth in Rust and emits compact metadata from the CLI.

## Goals / Non-Goals

**Goals:**
- Publish deterministic metadata for receipt/check-report schemas, version constants, required fields, compared fields, and non-claim boundaries.
- Validate the checked-in internal corpus receipt fixture against the same required-field/boundary expectations used by the check path.
- Fail closed for missing schema/version/boundary fields in tests.

**Non-Goals:**
- No JSON Schema dependency or generated schema language commitment.
- No HIR/MIR semantic hashing.
- No correctness, semantic-equivalence, or cross-machine determinism proof.

## Decisions

### 1. Rust-derived CLI metadata first

**Choice:** Add `cargo octet object corpus contract --output-format human|json`.
**Rationale:** The authoritative constants and DTO field knowledge already live in Rust; a CLI report makes them reproducible and easy to gate without adding a schema generator dependency.
**Alternative:** Hand-author Nickel or JSON Schema now. Rejected for this slice because it would risk drift from Rust DTOs before a generator exists.

**Implementation:** Add serializable contract metadata structs in `object.rs`, parser support in `cli.rs`, and tests that assert metadata matches emitted receipts/check reports.

### 2. Fixture validation through contract expectations

**Choice:** Tests validate the checked-in internal corpus receipt fixture against the metadata-required fields and boundary non-claims.
**Rationale:** This directly closes the current gap: fixture shape compatibility is enforced by the same Rust-owned metadata surface reviewers inspect.
**Alternative:** Only test the metadata command. Rejected because it would not prove the checked-in fixture is covered.

## Risks / Trade-offs

**Metadata drift** → Generate the report from the same constants and field lists used by validation/comparison helpers.

**Overclaiming schema strength** → The report is explicitly compatibility metadata, not a full schema language or proof of correctness/semantic equivalence.
