## Phase 1: Implementation

- [x] I1 Rename canonical Cargo package, binary, wrapper directory, Nix package/app outputs, and repo config surfaces from `tigerstyle` to `slotcar`.
- [x] I2 Update README, guides, templates, examples, and generated-output snapshots so current install/run/configure instructions use `slotcar` while preserving legitimate Tiger Style philosophy text.
- [x] I3 Implement or retain an explicit `cargo tigerstyle` compatibility alias only through the canonical `cargo slotcar` path.
- [x] I4 Add deterministic remaining-name classification support for non-archived `tigerstyle` occurrences.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Canonical install and usage docs say slotcar" with the remaining-name classification helper; save output at `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/name-classification.txt`.
- [x] V2 Verify scenario "Philosophy references remain distinguishable" with the remaining-name classification helper; save output at `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/name-classification.txt`.
- [x] V3 Verify scenario "Cargo subcommand uses slotcar" with a focused cargo-wrapper integration run; save output at `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`.
- [x] V4 Verify scenario "Structured outputs name slotcar" with focused human/JSON/SARIF output checks; save output at `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`.
- [x] V5 Verify scenario "Compatibility alias is explicit" with the compatibility invocation or a documented no-alias decision; save output at `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`.

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Canonical install and usage docs say slotcar` -> `V1` -> `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/name-classification.txt`
- `Scenario: Philosophy references remain distinguishable` -> `V2` -> `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/name-classification.txt`
- `Scenario: Cargo subcommand uses slotcar` -> `V3` -> `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`
- `Scenario: Structured outputs name slotcar` -> `V4` -> `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`
- `Scenario: Compatibility alias is explicit` -> `V5` -> `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt`
