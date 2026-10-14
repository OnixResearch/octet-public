## Phase 1: Implementation

- [x] I1 Add Rust manifest schema, deterministic emission, and artifact-backed validation
- [x] I2 Add Nickel contract/fixture coverage for the artifact integrity manifest
- [x] I3 Add CLI and Nix regression coverage for manifest export, tamper rejection, and intact-bundle acceptance

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Artifact export writes integrity manifest" with focused CLI/Nickel/Nix checks; save output at `openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/manifest-export.txt`
- [x] V2 Verify scenario "Artifact-backed check rejects tampered generated artifact" with focused CLI tests; save output at `openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/manifest-negative.txt`
- [x] V3 Verify scenario "Artifact-backed check accepts intact generated bundle" with focused/full standards checks; save output at `openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/manifest-positive.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`git diff -- crates/tigerstyle-standards/src/artifacts.rs crates/tigerstyle-standards/src/main.rs crates/tigerstyle-standards/tests/advisory_cli.rs standards/policies/contracts.ncl flake.nix`; input=`worktree diff`; asserts=`manifest validation fails before policy checks on missing, unsupported, or mismatched generated artifacts`; evidence=`openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/review-diff.txt`

## Change Status
- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Artifact export writes integrity manifest` -> `V1` -> `openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/manifest-export.txt`
- `Scenario: Artifact-backed check rejects tampered generated artifact` -> `V2` -> `openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/manifest-negative.txt`
- `Scenario: Artifact-backed check accepts intact generated bundle` -> `V3` -> `openspec/changes/archive/2026-05-06-enforce-generated-artifact-integrity-manifest/evidence/manifest-positive.txt`
