# valence-core

`valence-core` contains pure deterministic core logic that Octet hosts for bounded Valence evidence surfaces.

Current in-repository public modules:

- `evidence_codebase_index`: builds, queries, renders, and boundary-checks Octet-owned evidence-codebase indexes from in-memory artifacts; Octet retains CLI parsing, filesystem traversal, artifact discovery, output writing, Nix gate ownership, release bundle ownership, and non-claim boundary enforcement.
- `evidence_index`: renders and checks the current-head Valence evidence page from typed inputs; Octet retains command execution, repository-state capture, freshness orchestration, and publication decisions.
- `function_object`: constructs Function Object v1 DTOs, canonical hash payloads, caveat summaries, and pure-cache eligibility from source facts; Octet retains scanning, source-marker mutation, corpus orchestration, and CLI/status reporting.

## Boundary

This crate is intentionally an in-repository readiness seam, not a standalone release cutover or standalone repository cutover. It does not own CLI parsing, filesystem access, source scanning, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, process exits, repository tags, package publication, or release ownership. Those remain Octet-owned until a future reviewed cutover changes the contract.

## Evidence limits

The crate supports bounded local evidence replay for selected Octet/Valence surfaces. It does not claim behavioral correctness, semantic equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, external consumer compatibility, or general HIR/THIR/MIR extraction.
