# Promote Low-Noise Starter Lints

## Why

`new-lints-missing-principles` finished the first Aspen classification pass for
six new lint families. Four landed below the repo's low-noise threshold:

- `explicit_defaults` — 1 starter-scope finding, 2 `aspen-coordination` findings
- `unbounded_channel` — 0 / 0
- `usize_in_public_api` — 0 / 0
- `negated_predicate` — 0 / 2

The pinned Aspen phase-1 replay for this change stays within the same low-noise
threshold but shifts `usize_in_public_api` to `3 / 0` on commit `909e5e9fa`.
That still qualifies for starter promotion under the <=3 / <=5 rule.

The rollout kit still treats all six as pending and keeps them out of copied
starter-profile configs. That was correct before the classification review, but
it is now stale. Later consumers need the next documented step: promote the
four low-noise families into the starter profile while keeping the two noisy
families deferred.

## What Changes

- promote `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
  `negated_predicate` into the documented starter / boundary-first rollout
  profile
- keep `unbounded_collection_growth` and `too_many_parameters`
  inventory-first until a later cleanup review lands
- capture fresh Aspen evidence for the promoted starter profile using the
  first-class `cargo-tigerstyle` path, with saved artifacts under
  `openspec/changes/promote-low-noise-starter-lints/evidence/`
- save separate promoted-profile transcripts for both the starter scope and a
  focused `aspen-coordination` lib-only spotcheck that uses the same promoted
  lint set
- update rollout docs, README, and consumer template comments so copied starter
  configs match the promoted profile

## Capabilities

### Modified Capabilities

- `downstream-rollout-kit`: consumers can copy a starter profile that includes
  the four low-noise new lint families while still deferring the two noisy
  families

## Non-Goals

- changing lint implementations or thresholds
- promoting `unbounded_collection_growth` or `too_many_parameters`
- widening Aspen package scope beyond the documented starter packages from
  `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`:
  `aspen-time`, `aspen-hlc`, and `aspen-core`
- revisiting the archived classification thresholds from
  `2026-04-16-new-lints-missing-principles`

## Impact

- **Files**: `docs/downstream-rollout.md`, `README.md`, `templates/dylint.toml`,
  documentation OpenSpec delta/spec sync,
  `openspec/changes/promote-low-noise-starter-lints/evidence/*`
- **APIs**: no Rust API changes
- **Dependencies**: none
- **Testing**: fresh Aspen promoted-profile replay through
  `cargo-tigerstyle`, saved under
  `openspec/changes/promote-low-noise-starter-lints/evidence/` as:
  `replay-aspen-promoted-starter.sh`,
  `check-aspen-promoted-starter.py`,
  `aspen-promoted-starter-config.toml`,
  `aspen-promoted-starter-default.txt`,
  `aspen-promoted-starter-coordination.txt`, and
  `aspen-promoted-starter-counts.txt`
