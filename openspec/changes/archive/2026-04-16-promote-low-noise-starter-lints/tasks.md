# Tasks: Promote Low-Noise Starter Lints

## Phase 1: Spec and rollout docs

- [x] Extend the documentation spec so the rollout kit covers promotion of low-noise starter-candidate lint families into the starter profile
- [x] Update `docs/downstream-rollout.md` so starter / boundary-first examples enable `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and `negated_predicate`, while still deferring `unbounded_collection_growth` and `too_many_parameters`
- [x] Update `README.md` and `templates/dylint.toml` so consumer-facing starter guidance matches the promoted profile

## Phase 2: Aspen promotion evidence

- [x] Add a replay helper and checker for the promoted starter profile (`evidence/replay-aspen-promoted-starter.sh`, `evidence/check-aspen-promoted-starter.py`)
- [x] Capture self-describing Aspen transcripts for the promoted starter default run and a focused `aspen-coordination` spotcheck, plus the applied config snapshot
- [x] Save a count/assertion summary that proves the promoted starter profile and keeps `unbounded_collection_growth` plus `too_many_parameters` deferred
