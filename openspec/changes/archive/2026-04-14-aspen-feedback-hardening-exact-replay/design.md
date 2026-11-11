# Design: Prove The Exact Aspen Replay Path We Document

## Context

The repo now has a working Aspen phase-1 replay helper and a transcript saved
from the archived location. Gap: `docs/downstream-rollout.md` still centers the
manual sandbox sequence, while the proof we actually trust is the helper-driven
replay. The result is subtle but important: docs, transcript, and task claim are
not yet perfectly aligned.

A second issue is narrower: the count checker prints `aspen-hlc: 0 starter
findings` without asserting it. That weakens one of the summary points the
transcript claims to verify.

## Goals / Non-Goals

**Goals:**

- make the published Aspen replay path identical to the path the saved
  transcript runs
- assert every claimed summary point in the count checker, including
  `aspen-hlc: 0 starter findings`
- rerun archive-local artifacts so the saved transcript and verification notes
  point at the same exact helper path

**Non-Goals:**

- changing tigerstyle lint behavior
- changing Aspen rollout scope or counts
- replacing the helper with a fully manual copy-paste sequence

## Decisions

### 1. Document the helper as the exact replay path

**Choice:** Rewrite the Aspen reconstruction section so the first-class replay
path is the archived helper command.

**Rationale:** That is the path we can prove exactly. The manual sandbox steps
still matter as explanation, but the done review is right: the exact replay
claim should point to the actual helper/transcript pair.

**Alternative:** Keep the manual section as the primary path and try to record a
one-off transcript from hand-written shell commands. Rejected because the helper
already captures the working sequence exactly and is the cleaner audit target.

**Implementation:** Update the reconstruction section to present the helper
command, then summarize what the helper does internally.

### 2. Make the checker assert the HLC zero-findings claim

**Choice:** Assert both that `aspen-hlc` was checked and that it produced no
warning summary line.

**Rationale:** The docs claim zero starter findings, not merely that the crate
appeared somewhere in the build log.

**Alternative:** Drop the `aspen-hlc` claim from docs and verification.
Rejected because the archived phase-1 summary includes it and we can verify it
cheaply.

**Implementation:** Update `check-aspen-phase1-counts.py` and rerun the replay.

### 3. Keep archive-local reruns mandatory

**Choice:** Regenerate the replay transcript and count outputs from the archived
helper location, not from an active change path.

**Rationale:** This keeps evidence self-contained after archiving and avoids the
same stale-path regression again.

**Alternative:** Copy earlier `.txt` outputs forward. Rejected because real
reruns are clearer and safer.

## Risks / Trade-offs

**[Docs become more script-centric]** The reconstruction section now privileges
an archived helper command over a fully manual walkthrough.
**Mitigation:** Keep a short explanation of the sandbox layout and helper steps
under the exact replay command.

**[Checker becomes stricter]** If Aspen phase-1 evidence ever drifts, the helper
will fail faster.
**Mitigation:** That is desired; the stricter assertion is the point.
