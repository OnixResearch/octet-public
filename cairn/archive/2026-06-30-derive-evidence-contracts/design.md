## Context

Project guidance says runtime receipts and persisted evidence summaries should prefer Rust-owned DTOs with generated Nickel contracts. `Finding` and `ProvenanceRecord` are versioned emitted records, so they are good candidates for generated contracts.

The generation path should avoid runtime Nickel execution. Rust tests or build-time helpers can produce deterministic contract artifacts that policy validation checks for freshness.

## Approach

1. Pick a deterministic contract artifact location for generated evidence DTO contracts.
2. Add a Rust-owned schema source for `Finding` v2 and `ProvenanceRecord` that can render Nickel contracts without hand-authored field drift.
3. Generate contracts for required fields, optional fields, schema versions, evidence-tier discriminants, and hash string fields.
4. Register generated contracts in `standards/policies/registry.ncl` with `rust-derived` ownership and artifact freshness commands.
5. Add Nickel fixtures that exercise valid records and fail-closed bad records.

## Risks

- Rust derive reflection may not expose enough detail. Mitigate by using a small explicit Rust-owned schema table colocated with the DTOs.
- Generated Nickel may become noisy. Mitigate with stable ordering and committed golden artifacts.
- Contract strictness could reject forward-compatible fields. Mitigate by making the schema-version behavior explicit and documenting extension rules.

## Validation

- Positive: valid `Finding` v2 and `ProvenanceRecord` fixtures pass generated contracts.
- Negative: missing required fields, invalid evidence tier, unsupported schema version, and wrong field types fail with deterministic diagnostics.
- Freshness: policy validation fails when the committed generated contract does not match the Rust-owned schema source.
