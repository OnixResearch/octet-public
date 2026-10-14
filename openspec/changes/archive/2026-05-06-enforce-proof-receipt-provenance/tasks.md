# Enforce Proof Receipt Provenance Tasks

Change Status: archived
Open tasks: 0

## Implementation

- [x] I1 Add typed Nickel contract fields for proof receipt provenance.
- [x] I2 Add Rust model and generated artifact export support for proof receipt provenance.
- [x] I3 Validate proof receipt provenance lines in the verified proof rail checker.
- [x] I4 Generate proof receipt provenance lines from `--execute-proof-rails`.
- [x] I5 Update verification fixtures, invalid Nickel fixtures, tests, and docs.

## Verification

- [x] V1 Capture focused Rust proof/provenance checks. evidence=openspec/changes/archive/2026-05-06-enforce-proof-receipt-provenance/evidence/focused-checks.txt
- [x] V2 Capture generated artifact/provenance export checks. evidence=openspec/changes/archive/2026-05-06-enforce-proof-receipt-provenance/evidence/policy-artifact-export.txt
- [x] V3 Capture final validation transcript. evidence=openspec/changes/archive/2026-05-06-enforce-proof-receipt-provenance/evidence/final-validation.txt

## Hygiene

- [x] H1 Archive the completed OpenSpec and scrub stale active paths. command=openspec archive enforce-proof-receipt-provenance --yes input=completed tasks asserts=archived change has no active-path evidence links evidence=openspec/changes/archive/2026-05-06-enforce-proof-receipt-provenance/tasks.md
- [x] H2 Commit, push, and verify clean worktree/no active OpenSpecs. command=git status --short --branch && openspec list input=archived implementation asserts=HEAD matches origin/main and no active changes evidence=final response

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Proof receipt provenance obligations exported | policy-artifact-export.txt |
| Command-generated receipt includes provenance | focused-checks.txt |
| Stale receipt without provenance is rejected | focused-checks.txt |
| Final focused checks pass | final-validation.txt |
