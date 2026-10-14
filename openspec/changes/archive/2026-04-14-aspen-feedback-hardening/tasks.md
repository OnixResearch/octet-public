# Tasks: Harden Tigerstyle From Aspen Rollout Findings

## Phase 1: Reusable downstream kit

- [x] Document named starter rollout profiles, their intended first-use scope, and the promotion checklist for adding more crates/packages
- [x] Add a reusable downstream check-path recipe/template for nix-managed or otherwise nonstandard Dylint toolchains
- [x] Add a sample `dylint.toml` template with commented threshold, allowlist, and rationale sections for consumer-owned policy

## Phase 2: Guidance for findings Aspen exposed

- [x] Add `ambient_clock` boundary-pattern guidance for timeout/deadline helpers, elapsed-time measurement, and shell-owned wait loops
- [x] Add a high-noise lint promotion playbook covering `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, and `unbounded_loop`
- [x] Document known classification patterns and anti-patterns, including when narrow item-scoped `allow`s are acceptable and when blanket suppression is not

## Phase 3: Validation and evidence

- [x] Validate the new kit against Aspen's archived rollout evidence and update any gaps or contradictions it exposes
- [x] Capture evidence showing that a maintainer can reconstruct Aspen's phase-1 rollout path from the new `tigerstyle` guidance without relying on undocumented shell state
