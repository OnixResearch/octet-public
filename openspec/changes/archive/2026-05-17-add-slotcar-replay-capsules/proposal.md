## Why

Slotcar already emits machine-readable findings, fingerprints, review manifests, and example transcripts, but operators still need to manually assemble the exact command/config/input/output bundle that proves a run is replayable. Unison transcripts suggest the missing product shape: a first-class, deterministic replay capsule for `cargo slotcar check` and `cargo slotcar fingerprint` runs.

## What Changes

- Add a replay-capsule contract for lint and fingerprint runs.
- Require capsule inputs to pin repo revision, Slotcar binary identity, command argv, config hash, selected source hashes, baseline/fingerprint manifests, and expected output hashes.
- Require capsule replay to distinguish exact match, expected failure, and drift.
- Add operator documentation and saved evidence expectations for capsule capture/replay.

## Non-Goals

- Do not replace existing JSON, SARIF, baseline, or fingerprint outputs.
- Do not store raw secrets, absolute home paths, local usernames, or unbounded environment dumps.
- Do not claim semantic correctness from a replay match; the capsule proves reproducibility only.

## Capabilities

### New Capabilities
- `slotcar-replay-capsules`: Deterministic capture and replay contract for Slotcar check/fingerprint evidence.

### Modified Capabilities
- `consumer-integration`: Adds replay-capsule command/output behavior.
- `documentation`: Adds operator guidance for capsule evidence.

## Impact

- **Files**: future CLI DTOs, docs, examples, tests, and OpenSpec evidence fixtures.
- **APIs**: likely `cargo slotcar replay-capsule capture|replay|inspect` or equivalent subcommands.
- **Testing**: deterministic positive and negative replay fixtures with saved transcript artifacts.
