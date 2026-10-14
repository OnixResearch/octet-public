## Phase 1: Implementation

- [x] [serial] r[octet.structured_migration_sessions.receipt] Define structured migration session DTOs and receipt schema with name pointers, content identities, compatibility classification, affected surfaces, commands, status, and non-claims.
- [x] [serial] r[octet.structured_migration_sessions.plan] Add deterministic planning/status output for namespace, lint/policy, config, command, artifact-layout, and evidence-schema migration kinds.
- [x] [serial] r[octet.structured_migration_sessions.verify] Add verification that checks stale hashes, missing affected surfaces, alias coverage, artifact freshness, and explicit removal boundaries.
- [x] [serial] r[octet.structured_migration_sessions.apply] Add guarded apply support only for selected safe operations, with dry-run parity and pre-write operation manifests.
- [x] [serial] r[octet.structured_migration_sessions.boundaries] Add docs/examples that show a migration session from plan to verify while preserving archive and evidence boundaries.

## Phase 2: Verification

- [x] [serial] r[octet.structured_migration_sessions.receipt.valid] Verify a valid session receipt records old/new pointers, BLAKE3 identities, affected surfaces, compatibility classes, commands, and non-claims.
- [x] [serial] r[octet.structured_migration_sessions.receipt.invalid] Verify malformed receipts, missing identities, unknown compatibility classes, and unsupported migration kinds fail closed.
- [x] [serial] r[octet.structured_migration_sessions.verify.alias_coverage] Verify a namespace migration with canonical and compatibility aliases passes when all live surfaces are covered and fails when one alias or removal boundary is missing.
- [x] [serial] r[octet.structured_migration_sessions.verify.archive_boundary] Verify archive/read-only surfaces are reported and not rewritten unless explicitly classified as mutable.
- [x] [serial] r[octet.structured_migration_sessions.apply.dry_run_parity] Verify selected safe apply operations match dry-run plans, emit pre-write manifests, and remain idempotent.
- [x] [serial] r[octet.structured_migration_sessions.boundaries.no_semantic_equivalence] Verify docs and receipts avoid claims that name-pointer migration proves semantic equivalence or correctness.
- [x] [serial] Run Cairn validate plus proposal/design/tasks gates for this change package before implementation closeout. [depends:cairn-gates]
