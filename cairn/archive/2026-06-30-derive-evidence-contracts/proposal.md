## Why

Octet emits versioned evidence records from Rust DTOs, including finding sidecars and provenance records. The policy registry already treats these families as Rust-derived, but reviewers need generated contracts that make the wire shape explicit and freshness-checkable.

Generating Nickel contracts from the Rust-owned DTOs keeps runtime output and policy validation aligned without hand-maintaining parallel schemas.

## What Changes

- Generate deterministic Nickel contracts for `Finding` v2 and `ProvenanceRecord` from Rust-owned DTO definitions or a Rust-owned schema source.
- Register the generated contract artifacts and freshness checks in the policy/receipt family registry.
- Add positive and negative fixtures proving the contracts accept valid emitted records and reject missing fields, invalid evidence tiers, unsupported schema versions, and wrong field types.
- Document the ownership rule: Rust owns emitted receipt DTOs; Nickel validates generated contracts.

## Impact

- **Files**: `cargo-octet/src/output.rs`, `cargo-octet/src/evidence.rs`, generated contract location under `standards/policies/` or a dedicated evidence contract path, `standards/policies/registry.ncl`, policy validation tests, and contributor docs.
- **Testing**: contract generation freshness, positive fixture acceptance, and negative fixture rejection for both finding and provenance records.
