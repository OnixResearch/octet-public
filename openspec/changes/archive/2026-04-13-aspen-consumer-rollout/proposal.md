# Integrate Tigerstyle Into Aspen

## Why

`tigerstyle` now has the lint implementations, README, and baseline triage
for real consumer repos, but it still lacks a proven downstream rollout path.
`openspec/changes/custom-clippy-lints/tasks.md` still leaves two adoption
items open: wiring tigerstyle into aspen's flake checks and configuring
project-specific allowlists.

The aspen baseline is too noisy for a blind full-catalog rollout: 299
`ignored_result` matches, 421 `unwrap`/`expect`, 323 panic-family calls, 510
unchecked narrowings, and 173 unbounded loops. Shipping "enable everything"
as the integration story would produce noise, encourage blanket `allow`s, and
stall adoption.

## What Changes

- Define a staged downstream rollout plan for a real consumer workspace,
  starting with aspen.
- Require an explicit starter lint set instead of enabling the full catalog at
  once.
- Require per-project `dylint.toml` tuning and documented exceptions before CI
  promotion.
- Record validation evidence from an end-to-end aspen run so later consumers
  can copy a proven path.

## Capabilities

### New Capabilities

- `aspen-flake-check-integration`: aspen can run tigerstyle through its flake
  or CI check path
- `phased-consumer-rollout`: downstream repos can start with a low-noise lint
  subset and defer noisier families deliberately
- `per-project-lint-tuning`: consumer repos can tune thresholds and allowlists
  in `dylint.toml` with recorded rationale

## Impact

- **Files**: new openspec change here; later aspen `flake.nix`, aspen
  `dylint.toml`, and rollout notes or README updates
- **APIs**: no tigerstyle Rust API changes expected
- **Dependencies**: no new crate dependencies; uses existing dylint toolchain
- **Testing**: end-to-end aspen check run, baseline capture, config review
