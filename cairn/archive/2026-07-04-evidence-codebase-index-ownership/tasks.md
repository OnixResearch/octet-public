## Phase 1: Implementation

- [x] [serial] r[octet.evidence_codebase_index.ownership] Document the evidence-codebase index as an Octet-owned capability currently backed by a pure core hosted in `valence-core`.
- [x] [serial] r[octet.evidence_codebase_index.ownership.valence_hosted] Update `valence-core` README/API docs to state Octet-retained shell and release responsibilities for `evidence_codebase_index`.
- [x] [serial] r[octet.evidence_codebase_index.module_decomposition] Split the implementation into pure model, ingest, query, HTML, and boundary helpers while preserving existing public functions.
- [x] [serial] r[octet.evidence_codebase_index.compatibility] Preserve JSON schema, query output, HTML boundary output, deterministic sorting, and strict/advisory issue semantics.

## Phase 2: Verification

- [x] [serial] r[octet.evidence_codebase_index.ownership.output_boundary] Verify generated JSON/HTML/docs name the current capability as Octet-owned evidence indexing and do not imply standalone Valence release ownership.
- [x] [serial] r[octet.evidence_codebase_index.module_decomposition.public_api] Verify existing `build_evidence_codebase_index`, `query_evidence_index`, `render_evidence_index_html`, and `output_has_required_boundary` callers continue to compile unchanged.
- [x] [serial] r[octet.evidence_codebase_index.compatibility.byte_stable] Verify repeated builds over the fixture corpus remain byte-stable and query/HTML outputs preserve existing boundary text.
- [x] [serial] r[octet.evidence_codebase_index.compatibility.overclaim_negative] Verify negative tampered outputs that claim correctness, semantic equivalence, dependency-closure proof, remote-code-mobility guarantee, or standalone Valence release ownership fail the boundary check.
- [x] [serial] Run focused `valence-core` evidence-codebase tests, the named `evidence-codebase-index` Nix rail, `valence-core-package-readiness`, and Cairn validate/gates before implementation closeout. [depends:cairn-gates]
