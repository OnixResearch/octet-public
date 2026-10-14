## Why

`tigerstyle.embedded.bounded-queues` is now present in the generated policy artifact seam and Aspen consumes that seam, but the next enforcement step must prove more than metadata evaluation. Unbounded queues are a concrete Tiger Style resource hazard: they can turn backpressure bugs into latent OOMs. The standards checker should make this structural policy fail deterministically when configured forbidden APIs or missing bound witnesses appear, including in downstream consumer checks.

## What Changes

- Specify bounded-queue structural enforcement for generated policy artifacts and `tigerstyle-standards check`.
- Require the bounded primitive mechanism to export approved bounded APIs, forbidden unbounded constructors, and capacity/backpressure witness expectations.
- Require local positive/negative fixtures plus an Aspen scratch mutation proof for the downstream `checks.tigerstyle-policy-artifacts` gate.

## Non-Goals

- No broad type-system bounded queue library is required in this change; wrappers and approved APIs may be recognized before a reusable crate exists.
- No blanket ban on every channel-like abstraction is introduced without an explicit mechanism contract entry.
- No implementation is included in this spec-only package.

## Capabilities

### New Capabilities

- `bounded-queue-structural-gate`: Artifact-backed structural checks reject configured unbounded queue constructors and accept approved bounded primitives with explicit capacity/backpressure evidence.

### Modified Capabilities

- `standards-codification`: Structural mechanism validation becomes concrete for the first embedded bounded primitive policy.
- `consumer-integration`: Downstream Nix checks can prove a consumer cannot merge a configured unbounded queue drift silently.

## Impact

- **Files / artifacts**: `standards/policies/registry.ncl`, `crates/tigerstyle-standards/src/resource.rs`, resource fixtures under `standards/check-fixtures/`, JSON/TOML artifact export tests, `docs/examples/aspen-policy-artifact-dogfood.md`, and evidence under `openspec/changes/archive/2026-05-05-enforce-bounded-queue-structural-gate/evidence/`.
- **Ordering / dependencies**: generated artifact fields must be added before the checker can consume them; local fixtures should pass before Aspen scratch mutation proof is captured.
- **Verification / evidence**: `cargo test -p tigerstyle-standards`, `nix run .#tigerstyle-policy-artifacts`, `nix build .#checks.x86_64-linux.standards-fixtures --no-link -L`, `nix build .#checks.x86_64-linux.policy-artifact-consumer-fixture --no-link -L`, and an Aspen scratch-copy mutation transcript for `checks.x86_64-linux.tigerstyle-policy-artifacts`.
