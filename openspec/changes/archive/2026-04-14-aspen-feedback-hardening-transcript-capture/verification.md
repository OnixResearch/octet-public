# Verification: Prove The Transcript Capture Method We Claim

## Implementation Evidence

- Changed file: `AGENTS.md`
- Changed file: `.agent/napkin.md`
- Changed file: `docs/downstream-rollout.md`
- Changed file: `openspec/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/tasks.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/verification.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/check-aspen-phase1-counts.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/helper-script-syntax.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-change-tree.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-list.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-validate-specs.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-review.diff`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/prepare-aspen-phase1-worktree.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-default-run.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-coordination-spotcheck.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/task-checklist.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-dylint-parse.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-dylint-parse.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-script-check.txt`

## Task Coverage

- [x] Update `docs/downstream-rollout.md` so the saved Aspen transcript is regenerated through a documented archive-local capture helper, not an undocumented shell wrapper
  - Evidence: `docs/downstream-rollout.md`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.txt`
- [x] Extend the documentation spec so transcript preambles must come from a documented saved helper or exact capture command
  - Evidence: `openspec/specs/documentation/spec.md`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/specs/documentation/spec.md`
- [x] Add archive-local consistency checks that verify the capture helper, replay helper, and saved transcript all agree
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.txt`
- [x] Re-run the archive-local capture helper so `reconstruct-aspen-phase1.txt` is saved by the same documented command the docs reference
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.txt`
- [x] Refresh verification and task-checklist evidence so the Phase 2 completion claim is backed by the new capture helper and transcript
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/verification.md`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/task-checklist.txt`

## Validation Notes

- Docs now name one exact archive-local transcript-regeneration command:
  `bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh`.
- The saved transcript begins with that same command, followed by the wrapped
  replay-helper command. Both lines are emitted by the saved capture helper,
  not by an undocumented outer shell wrapper.
- `archive-consistency-check.py` now verifies the full chain:
  docs -> capture helper -> replay helper -> saved transcript.
- `check-aspen-phase1-counts.py` still explicitly asserts
  `aspen-hlc: 0 starter findings`.

## Archive Final State

- Change archived under `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/`.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-change-tree.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-list.txt`
- Documentation specs validate cleanly after sync.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-validate-specs.txt`
- Synced documentation-spec and implementation changes are captured in a reviewable diff.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-review.diff`

## Verification Commands

- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/helper-script-syntax.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash -n templates/tigerstyle-check.sh'; bash -n templates/tigerstyle-check.sh; echo '$ bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh'; bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh; echo '$ bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh'; bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh; echo '$ python3 -m py_compile openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/*.py'; python3 -m py_compile openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/*.py; echo 'ok'; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/helper-script-syntax.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-script-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash templates/tigerstyle-check.sh --help'; bash templates/tigerstyle-check.sh --help; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-script-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-dylint-parse.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-dylint-parse.py'; python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-dylint-parse.py; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/template-dylint-parse.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.py'; python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.py; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/archive-consistency-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.txt`
  - Command: `cd /path/to/git/tigerstyle && bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-default-run.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-coordination-spotcheck.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/task-checklist.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ sed -n "1,200p" openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/tasks.md'; sed -n '1,200p' openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/tasks.md; echo; echo 'Evidence map:'; echo '- Phase 1 task 1 -> docs/downstream-rollout.md, evidence/record-reconstruct-aspen-phase1.sh, evidence/reconstruct-aspen-phase1.txt'; echo '- Phase 1 task 2 -> openspec/specs/documentation/spec.md, specs/documentation/spec.md'; echo '- Phase 1 task 3 -> evidence/archive-consistency-check.py, evidence/archive-consistency-check.txt'; echo '- Phase 2 task 1 -> evidence/record-reconstruct-aspen-phase1.sh, evidence/reconstruct-aspen-phase1.txt'; echo '- Phase 2 task 2 -> verification.md, evidence/task-checklist.txt, evidence/reconstruct-aspen-phase1.txt'; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/task-checklist.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-validate-specs.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate --specs --strict --no-interactive > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-validate-specs.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-list.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec list > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-openspec-list.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-change-tree.txt`
  - Command: `cd /path/to/git/tigerstyle && find openspec/changes -maxdepth 2 -type d | sort > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-change-tree.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-review.diff`
  - Command: `cd /path/to/git/tigerstyle && git diff HEAD -- AGENTS.md .agent/napkin.md docs/downstream-rollout.md openspec/specs/documentation/spec.md openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/post-archive-review.diff`
