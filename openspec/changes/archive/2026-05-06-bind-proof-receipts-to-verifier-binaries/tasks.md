# Tasks: Bind proof receipts to verifier binaries

## Change Status

Complete; every implementation, verification, and hygiene task below is checked and evidence paths are populated.

## Implementation

- [x] I1 Add typed Nickel/Rust receipt verifier binary path/digest metadata.
- [x] I2 Enforce verifier binary receipt lines in receipt-only proof checks and command-generated receipts.
- [x] I3 Export verifier binary obligations in generated policy artifacts and update fixtures/docs.

## Verification

- [x] V1 Run focused Rust tests for verifier binary receipt rejection and generated receipts. evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/focused-checks.txt
- [x] V2 Run policy/artifact fixture checks that prove Nickel contracts and artifact export. evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/policy-artifact-export.txt
- [x] V3 Run final validation, OpenSpec validation, and whitespace hygiene. evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/final-validation.txt; evidence=openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/hygiene-after-trim.txt

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| r[verified-enforcement-rails.verifier-binary-accounting] | openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/focused-checks.txt |
| r[verified-enforcement-rails.generated-verifier-binary] | openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/focused-checks.txt |
| r[verified-enforcement-rails.export-verifier-binary] | openspec/changes/archive/2026-05-06-bind-proof-receipts-to-verifier-binaries/evidence/policy-artifact-export.txt |
