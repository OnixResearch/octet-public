## Phase 1: Implementation

- [x] [serial] r[octet.valence_evidence.release.bundle] Add a generated `valence-evidence-release` Nix gate that materializes a bundle index, the current-head evidence page, a freshness-check transcript, and BLAKE3 artifact hashes.
- [x] [serial] r[octet.valence_evidence.release.check] Verify the generated bundle index schema, artifact list, artifact hashes, freshness pass lines, and bounded non-claim list inside the derivation.
- [x] [serial] r[octet.valence_evidence.release.aggregate] Wire the release gate into `valence-object-evidence` with a visible aggregate PASS line.

## Phase 2: Verification

- [x] [serial] r[octet.valence_evidence.release.bundle] Run `nix build .#checks.x86_64-linux.valence-evidence-release --no-link -L` and confirm the bundle/hash/freshness PASS lines.
- [x] [serial] r[octet.valence_evidence.release.aggregate] Run `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L` and confirm `valence_object_evidence.evidence_release_gate: PASS`.
- [x] [serial] r[octet.valence_evidence.release.check] Run `nix run .#cairn -- validate --root .`, all three Cairn gates for this change, and `git diff --check`.
