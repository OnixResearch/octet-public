## Phase 1: Export core

- [x] [serial] r[octet.evidence_provenance.valence_export] Define the normalized Valence provenance export DTO for Octet provenance records, proof cards, receipts, artifact identities, parent links, and diagnostics.
- [x] [serial] r[octet.evidence_provenance.valence_export] Implement pure translation from verified Octet artifact records into the export DTO while preserving evidence tiers.
- [x] [serial] r[octet.evidence_provenance.valence_export.fail_closed] Add validation that blocks exports with stale hashes, missing parents, unknown checker IDs, missing proof receipts, or tier promotion.

## Phase 2: CLI shell

- [x] [serial] r[octet.evidence_provenance.valence_export.cli] Add `cargo octet artifact verify --emit-valence-provenance <path>` or an equivalent `cargo octet provenance export --out <path>` shell.
- [x] [serial] r[octet.evidence_provenance.valence_export.cli] Ensure the shell writes the export only after artifact verification succeeds for the referenced artifact directory.
- [x] [parallel] r[octet.evidence_provenance.valence_export.cli] Add CLI diagnostics that identify stale artifact hashes, missing parents, manual-review-only evidence, and proof-card/receipt linkage failures.

## Phase 3: Tests and docs

- [x] [parallel] r[octet.evidence_provenance.valence_export] Add positive tests for static-lint/policy evidence export and validated proof-card export.
- [x] [parallel] r[octet.evidence_provenance.valence_export.fail_closed] Add negative tests for artifact verification failure, missing parent, stale BLAKE3, unknown checker ID, missing proof receipt, and imported Trellis evidence promoted to `formal_proof`.
- [x] [serial] r[octet.evidence_provenance.valence_export.boundary] Document that Valence consumes the export for stack verification and Mantle bundles only the final sidecar as opaque external evidence.
- [x] [serial] r[octet.evidence_provenance.valence_export] Run focused validation: artifact verification tests, provenance export tests, `cargo fmt --check`, and relevant Cairn validation/gates.
