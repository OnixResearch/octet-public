## Phase 1: Implementation

- [x] [serial] r[octet.small_checker_receipts.producer_checker] Inventory receipt and provenance families where producer identity and checker identity are distinct or should become distinct.
- [x] [serial] r[octet.small_checker_receipts.producer_checker] Extend selected receipt/provenance DTOs or summaries with producer/checker identity while preserving compatibility or naming schema-version changes.
- [x] [serial] r[octet.small_checker_receipts.deterministic_checker] Extract or document small pure checker cores for selected proof receipt and artifact verification decisions.
- [x] [serial] r[octet.small_checker_receipts.registry] Register checker IDs, receipt families, generated contracts, and non-candidate raw internals in the policy registry.
- [x] [serial] r[octet.small_checker_receipts.boundaries] Update docs to state what checker acceptance does and does not prove.

## Phase 2: Verification

- [x] [serial] r[octet.small_checker_receipts.producer_checker.valid] Verify a valid producer/checker receipt validates and reports both identities deterministically.
- [x] [serial] r[octet.small_checker_receipts.producer_checker.self_asserted] Verify producer-only self-asserted evidence cannot be promoted to a checker-validated or `formal_proof` status.
- [x] [serial] r[octet.small_checker_receipts.deterministic_checker.fail_closed] Verify stale hashes, missing bindings, unsupported schemas, malformed receipts, unknown checker IDs, and overbroad claims fail closed with stable diagnostics.
- [x] [serial] r[octet.small_checker_receipts.registry.fresh] Verify generated contracts and registry entries stay fresh after DTO changes.
- [x] [serial] Run focused receipt/artifact verifier tests plus Cairn validate and proposal/design/tasks gates. [depends:cairn-gates]
