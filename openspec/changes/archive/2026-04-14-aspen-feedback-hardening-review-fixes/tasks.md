# Tasks: Fix Review Gaps In Aspen Feedback Hardening

## Phase 1: Assumptions and template alignment

- [x] Remove the template's avoidable `realpath` dependency and document the remaining required commands/environment assumptions accurately
- [x] Update the rollout guide so its minimum-assumption section matches the published template behavior exactly

## Phase 2: Audit-ready evidence

- [x] Store the full helper scripts used for validation as reviewable evidence files
- [x] Capture an end-to-end Aspen phase-1 reconstruction transcript using the new templates in a disposable worktree
- [x] Refresh verification notes so every evidence artifact points to a concrete command or stored helper script instead of elided here-docs
