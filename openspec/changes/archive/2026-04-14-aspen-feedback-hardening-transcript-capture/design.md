# Design: Prove The Transcript Capture Method We Claim

## Context

Current docs say the saved Aspen transcript comes from a direct shell
redirection of `reconstruct-aspen-phase1.sh`. Real artifact disagrees: the
transcript begins with a `$ bash ...` line that only appeared because an outer
shell wrapper echoed it before invoking the helper. Review is right: that
capture method was real, but undocumented and therefore not proven.

## Goals / Non-Goals

**Goals:**

- make one archive-local helper the documented transcript-regeneration command
- ensure the saved transcript preamble is emitted by that helper itself
- teach consistency checks to verify both the documented command string and the
  transcript preamble source

**Non-Goals:**

- changing Aspen rollout counts or helper internals
- changing tigerstyle lint behavior
- reintroducing the older manual sandbox walkthrough as the primary proof path

## Decisions

### 1. Separate replay helper from transcript-capture helper

**Choice:** Keep `reconstruct-aspen-phase1.sh` as the replay helper that does
sandbox setup and validation, then add `record-reconstruct-aspen-phase1.sh` as
one explicit wrapper that prints the top-level archive-local command and saves
`reconstruct-aspen-phase1.txt`.

**Rationale:** This removes the hidden outer wrapper. Reviewers can inspect the
capture helper directly and see exactly why the transcript starts with the
command line it does.

**Alternative:** Make `reconstruct-aspen-phase1.sh` self-recording. Rejected
because it would mix transcript capture concerns into the replay helper and make
ad-hoc stdout use harder.

### 2. Document the capture helper as the transcript source of truth

**Choice:** Update `docs/downstream-rollout.md` and verification notes to point
at `record-reconstruct-aspen-phase1.sh` when claiming how the saved transcript
was produced.

**Rationale:** The evidence defect is about capture method, not replay logic.
The documented command must match the saved artifact generation method exactly.

### 3. Verify transcript preamble provenance

**Choice:** Extend `archive-consistency-check.py` so it checks all three links:

- docs mention the archive-local capture helper command
- capture helper contains the exact printed preamble and calls the replay helper
- saved transcript starts with that same printed preamble

**Rationale:** Previous consistency checks only compared docs and helper text.
They did not prove that the transcript itself came from the documented capture
path.

## Risks / Trade-offs

**[One more helper file]** Evidence directory gains another script.
**Mitigation:** Keep roles narrow: one script replays, one script records.

**[Docs get more precise]** Readers now see transcript regeneration as a helper
command, not a bare shell redirection.
**Mitigation:** Keep replay-helper behavior summary below the exact command so
reviewers still see sandbox and symlink details.
