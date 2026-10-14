# Tasks: Bind proof receipts to build closure identity

## Change Status

Complete; every implementation, verification, and hygiene task below is checked and evidence paths are populated.

## Implementation

- [x] I1 Add typed Nickel/Rust receipt verifier build-closure metadata.
- [x] I2 Enforce build-closure receipt lines in receipt-only proof checks and command-generated receipts.
- [x] I3 Export build-closure obligations in generated policy artifacts and update fixtures/docs.

## Verification

- [x] V1 Run focused Rust tests for build-closure receipt rejection and generated receipts. evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-build-closure/evidence/focused-checks.txt
- [x] V2 Run policy/artifact fixture checks that prove Nickel contracts and artifact export. evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-build-closure/evidence/policy-artifact-export.txt
- [x] V3 Run final validation, OpenSpec validation, and whitespace hygiene. evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-build-closure/evidence/final-validation.txt

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| r[verified-enforcement-rails.build-closure-accounting] | openspec/changes/archive/2026-05-06-bind-proof-receipts-to-build-closure/evidence/focused-checks.txt |
| r[verified-enforcement-rails.generated-build-closure] | openspec/changes/archive/2026-05-06-bind-proof-receipts-to-build-closure/evidence/focused-checks.txt |
| r[verified-enforcement-rails.export-build-closure] | openspec/changes/archive/2026-05-06-bind-proof-receipts-to-build-closure/evidence/policy-artifact-export.txt |
