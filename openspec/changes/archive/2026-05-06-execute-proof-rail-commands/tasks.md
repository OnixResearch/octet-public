# execute-proof-rail-commands Tasks

## Change Status

- Status: archived
- Open tasks: none; implementation, evidence, archive, validation, commit/push prep completed

## Implementation

- [x] I1: Add opt-in proof command execution to `tigerstyle-standards check` while preserving read-only receipt checks by default.
- [x] I2: Update verification proof rail policy/fixtures so the configured command runs from the checked source root and emits the declared pass signal only for compliant fixtures.
- [x] I3: Export/document proof command execution obligations in generated artifacts and policy contract docs.

## Verification

- [x] V1: Capture policy artifact export evidence showing proof command metadata remains deterministic. evidence=`openspec/changes/archive/2026-05-06-execute-proof-rail-commands/evidence/policy-artifact-export.txt`
- [x] V2: Capture executed proof command fixture evidence proving compliant success and noncompliant failure. evidence=`openspec/changes/archive/2026-05-06-execute-proof-rail-commands/evidence/proof-command-fixtures.txt`
- [x] V3: Capture focused Rust/Nix/OpenSpec validation evidence. evidence=`openspec/changes/archive/2026-05-06-execute-proof-rail-commands/evidence/focused-checks.txt`

## Hygiene

- [x] H1: Archive the completed change, scrub stale active paths, validate specs, commit, push, and update the Tigerstyle skill reference. command=`openspec archive execute-proof-rail-commands --yes`; input=`completed tasks and evidence`; asserts=`archive status, no active change, validations pass`; evidence=`openspec/changes/archive/2026-05-06-execute-proof-rail-commands/evidence/focused-checks.txt`

## Verification Coverage

| Scenario | Evidence |
| --- | --- |
| Executed proof commands are opt-in and read-only checks remain supported | `proof-command-fixtures.txt`, `focused-checks.txt` |
| Compliant fixture command output supplies the pass receipt | `proof-command-fixtures.txt` |
| Noncompliant fixture command output fails the verified gate | `proof-command-fixtures.txt` |
| Generated artifacts keep proof command metadata deterministic | `policy-artifact-export.txt` |
