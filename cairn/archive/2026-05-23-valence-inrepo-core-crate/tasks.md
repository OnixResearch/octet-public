# Tasks

- [x] [serial] r[octet.valence_project_extraction.phase1.inrepo_core_crate] Add a workspace `valence-core` crate for pure deterministic Valence evidence logic.
- [x] [serial] r[octet.valence_project_extraction.phase1.evidence_index_core] Move evidence-index render/check core logic into `valence-core` while preserving existing output and fail-closed issue semantics.
- [x] [serial] r[octet.valence_project_extraction.phase1.cli_shell] Keep `cargo-octet` as the CLI/filesystem/stdout shell delegating to `valence-core`.
- [x] [serial] r[octet.valence_project_extraction.phase1.gates] Verify focused Rust tests, current evidence-index command behavior, named Valence Nix gates, and Cairn validation.
- [x] [serial] r[octet.valence_project_extraction.phase1.non_claims] Preserve bounded extraction non-claims and avoid standalone repo/package/release cutover.
