## Phase 1: Planning

- [x] [serial] r[octet.valence_project_extraction.phase0.inventory] Inspect current Octet/Valence code, fixture, docs, Cairn, and Nix surfaces and record the Phase 0 inventory artifact.
- [x] [serial] r[octet.valence_project_extraction.phase0.boundary] Define candidate Valence-owned, Octet-retained, and shared handoff surfaces without moving code.
- [x] [serial] r[octet.valence_project_extraction.phase0.gates] Define the named Nix/Cairn handoff gates that must survive a future extraction.
- [x] [serial] r[octet.valence_project_extraction.phase0.non_claims] Preserve bounded non-claims for the extraction plan.

## Phase 2: Verification

- [x] [serial] r[octet.valence_project_extraction.phase0.inventory] Review `evidence/phase0-inventory.md` against repository paths and current Nix check names.
- [x] [serial] r[octet.valence_project_extraction.phase0.gates] Run `nix run .#cairn -- validate --root .` and all three Cairn gates for this change.
- [x] [serial] r[octet.valence_project_extraction.phase0.non_claims] Run `git diff --check` and confirm the package does not introduce code movement or stronger Valence readiness claims.
