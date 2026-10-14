# Tasks: Prove The Transcript Capture Method We Claim

## Phase 1: Docs and evidence contract

- [x] Update `docs/downstream-rollout.md` so the saved Aspen transcript is regenerated through a documented archive-local capture helper, not an undocumented shell wrapper
- [x] Extend the documentation spec so transcript preambles must come from a documented saved helper or exact capture command
- [x] Add archive-local consistency checks that verify the capture helper, replay helper, and saved transcript all agree

## Phase 2: Archive-local reruns

- [x] Re-run the archive-local capture helper so `reconstruct-aspen-phase1.txt` is saved by the same documented command the docs reference
- [x] Refresh verification and task-checklist evidence so the Phase 2 completion claim is backed by the new capture helper and transcript
