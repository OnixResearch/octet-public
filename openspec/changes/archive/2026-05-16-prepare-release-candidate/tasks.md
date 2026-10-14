## Phase 1: Implementation

- [x] I1 Add CLI root version output for release smoke assertions
- [x] I2 Add release candidate checklist/receipt documentation
- [x] I3 Add Nix release candidate smoke check for package outputs and binary version surfaces

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Maintainer verifies package outputs before tagging" with release candidate smoke check; save output at `openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/release-candidate-smoke.txt`
- [x] V2 Verify scenario "Installed binary reports candidate version" with direct built-binary version commands; save output at `openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/version-smoke.txt`
- [x] V3 Verify scenario "Release candidate receipt is saved as review evidence" with evidence inventory; save output at `openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/evidence-inventory.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review docs/release-candidate.md and smoke evidence before tagging`; input=`docs/release-candidate.md and openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/*.txt`; asserts=`the release-candidate path is a pre-tag checklist and receipt, not a publish/tag side effect`; evidence=`openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/release-candidate-review.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Maintainer verifies package outputs before tagging` -> `V1` -> `openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/release-candidate-smoke.txt`
- `Scenario: Installed binary reports candidate version` -> `V2` -> `openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/version-smoke.txt`
- `Scenario: Release candidate receipt is saved as review evidence` -> `V3` -> `openspec/changes/archive/2026-05-16-prepare-release-candidate/evidence/evidence-inventory.txt`
