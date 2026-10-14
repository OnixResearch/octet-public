# Design: Capture Aspen's Phase-2 Widening In The Rollout Kit

## Context

The repo already preserves Aspen phase 1 well: starter profile, archived
counts, reconstruction helper, and consumer-owned tuning guidance. What it does
not preserve is what happened next. Aspen fixed the deferred coordination slice,
expanded the default package scope, and added two more low-noise lint families.
That promotion path is exactly what future consumers will ask for after their
first green starter run.

## Goals / Non-Goals

**Goals:**

- keep phase 1 as the canonical starter reference
- add a distinct phase-2 widening reference after the starter path
- show the concrete promotion inputs: deferred-package inventory, cleanup
  themes, widened default scope, and the widened lint set
- store fresh tigerstyle-side evidence rather than relying only on Aspen-local
  archived notes

**Non-Goals:**

- editing Aspen source again
- replaying Aspen's historical phase-1 commit
- promoting every high-noise family into the default kit

## Decisions

### 1. Treat widening as a separate documented phase

**Choice:** Add an explicit phase-2 widening section to
`docs/downstream-rollout.md` instead of folding it into the starter profiles.

**Rationale:** Starter rollout and widening have different questions. Phase 1
proves toolchain wiring and first signal. Phase 2 proves a deferred package can
rejoin the default path after targeted cleanup.

### 2. Capture fresh evidence from Aspen HEAD in a disposable sandbox

**Choice:** Run Aspen's current `scripts/tigerstyle-check.sh` from a temporary
sandbox parent with sibling symlinks for `aspen-dns` and `aspen-wasm-plugin`,
then save the widened default-pilot and targeted coordination transcripts under
this tigerstyle change.

**Rationale:** It avoids mutating the live Aspen checkout while keeping the
widened reference reproducible.

## Risks / Trade-offs

**[Aspen HEAD may keep evolving]** Future Aspen defaults can widen again.
**Mitigation:** describe this as the current phase-2 reference and keep the
starter path anchored to the archived phase-1 proof.
