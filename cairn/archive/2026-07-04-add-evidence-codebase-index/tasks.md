## Phase 1: Implementation

- [x] [serial] r[octet.evidence_codebase_index.model] Define pure DTOs and normalization for indexable artifact records, stable IDs, parent links, caveats, and BLAKE3 identities.
- [x] [serial] r[octet.evidence_codebase_index.ingest] Add artifact-family ingestion for current Octet evidence surfaces without changing their source formats.
- [x] [serial] r[octet.evidence_codebase_index.query] Add deterministic query/report commands for requirement IDs, lint IDs, policy IDs, fingerprints, artifact hashes, and provenance records.
- [x] [serial] r[octet.evidence_codebase_index.html] Add static HTML rendering from the same normalized index model.
- [x] [serial] r[octet.evidence_codebase_index.boundaries] Add generated and documented non-claim boundaries for index output.

## Phase 2: Verification

- [x] [serial] r[octet.evidence_codebase_index.model.deterministic] Verify repeated index builds over the same fixture emit byte-stable JSON and matching index hashes.
- [x] [serial] r[octet.evidence_codebase_index.ingest.malformed] Verify missing, malformed, stale-hash, and unsupported-family artifacts fail closed or produce explicit caveats without panicking.
- [x] [serial] r[octet.evidence_codebase_index.query.lookup] Verify queries find linked requirement, lint, policy, finding, fingerprint, provenance, and replay capsule records in a multi-artifact fixture.
- [x] [serial] r[octet.evidence_codebase_index.html.review_surface] Verify generated HTML contains deterministic links and no unsupported correctness or semantic-equivalence claims.
- [x] [serial] Run Cairn validate plus proposal/design/tasks gates for this change package before implementation closeout. [depends:cairn-gates]
