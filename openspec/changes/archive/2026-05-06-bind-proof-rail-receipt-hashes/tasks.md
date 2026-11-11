# bind-proof-rail-receipt-hashes Tasks

## Change Status

- Status: archived
- Open tasks: none

## Implementation

- [x] I1: Add receipt-binding metadata to the Nickel proof-rail contract, Rust model, generated artifacts, and verification registry policy.
- [x] I2: Generate command/source BLAKE3 binding lines in `--execute-proof-rails` receipts.
- [x] I3: Validate required receipt binding lines in the verified proof rail checker and emit hard diagnostics on missing/mismatched bindings.
- [x] I4: Refresh verification fixtures and CLI tests for passing generated receipts, stale static receipts, and source drift.

## Verification

- [x] V1: Capture focused Rust/CLI proof receipt binding checks. evidence=`openspec/changes/archive/2026-05-06-bind-proof-rail-receipt-hashes/evidence/focused-checks.txt`
- [x] V2: Capture generated policy artifact assertions for receipt binding metadata. evidence=`openspec/changes/archive/2026-05-06-bind-proof-rail-receipt-hashes/evidence/policy-artifact-export.txt`
- [x] V3: Capture full standards/docs/OpenSpec validation. evidence=`openspec/changes/archive/2026-05-06-bind-proof-rail-receipt-hashes/evidence/final-validation.txt`

## Hygiene

- [x] H1: Archive the OpenSpec, update skill notes, commit, push, and verify no active changes remain.

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Command and source hash-bound receipt accepted | V1 |
| Stale or mismatched receipt binding rejected | V1 |
| Generated artifacts expose receipt binding mode | V2 |
| Full project gates remain green | V3 |
