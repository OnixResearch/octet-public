# Prove The Rollout Kit On A Second Consumer

## Why

The rollout kit has Aspen-backed templates and cautions, but the repo still
lacks a non-Aspen replay showing another consumer can copy the templates,
choose a starter scope, and run the documented check path. Without that proof,
review can still dismiss the kit as Aspen-specific.

## What Changes

- pick one non-Aspen Rust consumer and run the kit against it from a disposable
  sandbox
- save the copied template files, the exact replay helper, and the resulting
  check transcript
- update the docs so the second-consumer proof is easy to find

## Capabilities

### Modified Capabilities

- `downstream-rollout-kit`: kit now has a non-Aspen proof that the templates
  and docs work on another real workspace

## Impact

- **Files**: rollout docs, OpenSpec docs/specs, second-consumer helper and transcript
- **APIs**: no Rust API changes
- **Dependencies**: no new Rust crates
- **Testing**: run the copied template against a second consumer from a clean sandbox
