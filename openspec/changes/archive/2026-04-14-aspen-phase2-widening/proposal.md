# Capture Aspen's Phase-2 Widening In The Rollout Kit

## Why

`tigerstyle` docs stop at Aspen's phase-1 starter rollout even though Aspen has
already finished the next step: `aspen-coordination` rejoined the default pilot
and Aspen's default lint set widened beyond the initial three-lint boundary-
first profile. Later consumers need that widening story, not only the phase-1
deferral snapshot.

## What Changes

- document Aspen's widened default pilot as the phase-2 reference after the
  archived phase-1 starter path
- add a rollout playbook for graduating one deferred package into the default
  path after targeted cleanup
- capture fresh reviewable evidence from Aspen HEAD showing the widened default
  scope and the cleaned `aspen-coordination` slice

## Capabilities

### Modified Capabilities

- `downstream-rollout-kit`: consumers can now see both the starter path and the
  first widening step that Aspen proved in practice

## Impact

- **Files**: `docs/downstream-rollout.md`, `README.md`, OpenSpec docs/specs,
  new evidence helpers/transcripts
- **APIs**: no Rust API changes
- **Dependencies**: no new Rust crates
- **Testing**: replay Aspen widening checks from a sandboxed Aspen HEAD worktree
