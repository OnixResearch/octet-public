# Harden Tigerstyle From Aspen Rollout Findings

## Why

The first Aspen rollout proved `tigerstyle` can run in a real consumer
workspace, but it also exposed gaps we should fix before asking a second repo
to adopt it.

Aspen's initial baseline was too noisy for a full-catalog rollout: 299
`ignored_result`, 421 `unwrap`/`expect`, 323 panic-family calls, 510
unchecked narrowings, and 173 unbounded loops. Even the focused
`aspen-coordination` spot check still surfaced 21 `ambient_clock`, 2
`compound_assertion`, and 4 `no_panic` warnings. On top of that, Aspen's
nix-managed nightly required a custom `scripts/tigerstyle-check.sh` wrapper to
patch Dylint driver/toolchain behavior.

Those findings are useful product feedback, not just Aspen-local cleanup.
Without a reusable onboarding kit, better `ambient_clock` boundary guidance,
and a playbook for high-noise lint families, future consumers will repeat the
same trial-and-error.

## What Changes

- Turn Aspen-only rollout lessons into reusable downstream onboarding guidance.
- Define named starter rollout profiles, config templates, and pilot-promotion
  checklists.
- Document `ambient_clock` boundary patterns seen in timeout/deadline helpers,
  retry loops, and worker timing code.
- Document an inventory-first promotion workflow for high-noise lint families
  before they enter the default downstream path.

## Capabilities

### New Capabilities

- `consumer-integration-kit`: later consumers can copy a documented,
  evidence-backed adoption path instead of rebuilding Aspen's wrapper/config
  story from scratch
- `ambient-clock-boundary-playbook`: maintainers can classify clock reads as
  core-logic violations vs legitimate shell-owned boundaries
- `high-noise-lint-promotion-playbook`: maintainers can promote noisy lint
  families with inventories and evidence instead of blanket `allow`s

## Impact

- **Files**: new downstream rollout docs/templates, updated README and spec
  guidance, possible reusable helper-script template
- **APIs**: no Rust API changes expected
- **Dependencies**: no new runtime dependencies expected
- **Testing**: validate new guidance against Aspen rollout artifacts and ensure
  the documented path reproduces Aspen's phase-1 setup
