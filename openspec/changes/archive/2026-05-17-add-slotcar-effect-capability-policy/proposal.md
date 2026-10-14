## Why

Unison abilities make effect requirements explicit. Slotcar should not implement algebraic effects, but it can make Rust effect and capability boundaries reviewable through policy metadata, lint families, and effect-surface receipts.

## What Changes

- Define the Slotcar capability vocabulary for time, randomness, filesystem, network, process, environment, secrets, remote execution, unsafe, global state, and concurrency.
- Require lint/policy outputs to map findings to capability families when the mapping is known.
- Require explicit unknown-capability behavior so new lints do not silently disappear from review surfaces.
- Require FCIS guidance to classify core-prohibited ambient effects and shell-allowed capabilities.

## Non-Goals

- Do not add a Rust language-level effect system.
- Do not require all Rust functions to carry source annotations.
- Do not block consumers on advisory capability families before fixture evidence exists.

## Capabilities

### New Capabilities
- `slotcar-effect-capability-policy`: Explicit capability/effect vocabulary and reporting obligations.

### Modified Capabilities
- `standards-codification`: Adds policy registry and checker mapping requirements.
- `consumer-integration`: Adds effect-family output behavior.

## Impact

- **Files**: future policy registry data, lint metadata, docs, JSON/SARIF output tests.
- **APIs**: output fields for capability/effect families and unknown-family diagnostics.
- **Testing**: positive mappings, unknown mappings, and FCIS boundary fixtures.
