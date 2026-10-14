# Tasks: Integrate Tigerstyle Into Aspen

## Phase 1: Rollout baseline

- [x] Choose the initial aspen starter lint set from existing triage evidence
- [x] Record which lint families and packages are deferred in phase 1 and why
- [x] Define promotion criteria for later rollout phases
- [x] Draft rollout notes that map aspen hotspot categories to rollout phases

## Phase 2: Aspen integration

- [x] Add tigerstyle invocation to aspen's `flake.nix` or equivalent check path
- [x] Add aspen `dylint.toml` with workspace-specific thresholds and allowlists
- [x] Record rationale for each committed threshold, allowlist, or explicit exception
- [x] Document any toolchain or environment requirements needed for the aspen check to run

## Phase 3: Validation evidence

- [x] Run the configured aspen check end to end and capture command, result summary, and representative findings
- [x] Confirm the starter lint set produces actionable signal without immediate blanket suppression
- [x] File follow-up changes for noisy lint families or tigerstyle false positives that block later promotion
