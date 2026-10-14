## Design

### 1. Release bundle is a Nix derivation, not a hand-maintained artifact

**Choice:** Add a `valenceEvidenceReleaseGate` derivation in `flake.nix`. The derivation builds from the flake source, runs `cargo-octet evidence-index check`, copies `docs/current-head-valence-evidence.md`, writes `valence-evidence-release-index.json`, and verifies all indexed BLAKE3 hashes before succeeding.

**Covers:** r[octet.valence_evidence.release.bundle]

**Rationale:** Keeping the bundle generated inside the check avoids stale checked-in release JSON while still producing a portable output under the Nix store for local audit.

**Boundary:** This is an Octet/Valence release-evidence rail, not a full Cairn release-readiness claim. It must not claim correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.

### 2. Freshness check transcript is included as evidence

**Choice:** The bundle stores `evidence-index-check.txt` and requires the pass lines `valence_evidence_index.valid: PASS` and `valence_evidence_index.issue_count=0`.

**Covers:** r[octet.valence_evidence.release.check]

**Rationale:** The transcript makes the bundle independently reviewable and gives the release index a concrete artifact proving the checked-in evidence page was fresh when the derivation ran.

### 3. Aggregate gate composes the release gate

**Choice:** `valence-object-evidence` depends on the new `valence-evidence-release` derivation and emits a dedicated `valence_object_evidence.evidence_release_gate: PASS` line.

**Covers:** r[octet.valence_evidence.release.aggregate]

**Rationale:** Operators already use `valence-object-evidence` as the selected Valence closeout rail; adding the release gate there prevents bypassing the portable bundle check.

## Deterministic Proof Plan

- `nix build .#checks.x86_64-linux.valence-evidence-release --no-link -L` must pass and print the release bundle/hash/check PASS lines.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L` must pass and include `valence_object_evidence.evidence_release_gate: PASS`.
- `nix run .#cairn -- validate --root .` plus proposal/design/tasks gates must pass for this change package.
