## Phase 1: Implementation

- [x] I1 Add typed proof receipt verifier/toolchain metadata to Nickel and Rust models
- [x] I2 Add verifier/toolchain metadata validation and command-generated receipt lines
- [x] I3 Export verifier/toolchain receipt fields through generated policy artifacts
- [x] I4 Update fixtures, CLI regressions, docs, and Nix assertions

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Receipt-only checks reject missing verifier/toolchain digests" with focused Rust CLI tests; save output at `openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-digests/evidence/focused-checks.txt`
- [x] V2 Verify scenario "Generated artifacts expose verifier/toolchain digest obligations" with policy artifact export; save output at `openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-digests/evidence/policy-artifact-export.txt`
- [x] V3 Verify scenario "Verified receipts account for verifier and toolchain identity" with final validation suite; save output at `openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-digests/evidence/final-validation.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Verified receipts account for verifier and toolchain identity` -> `V3` -> `openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-digests/evidence/final-validation.txt`
- `Scenario: Receipt-only checks reject missing verifier/toolchain digests` -> `V1` -> `openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-digests/evidence/focused-checks.txt`
- `Scenario: Generated artifacts expose verifier/toolchain digest obligations` -> `V2` -> `openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-digests/evidence/policy-artifact-export.txt`
