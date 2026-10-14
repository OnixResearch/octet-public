# Verification: Align Aspen Reconstruction Docs And Archived Evidence

## Implementation Evidence

- Changed file: `AGENTS.md`
- Changed file: `.agent/napkin.md`
- Changed file: `docs/downstream-rollout.md`
- Changed file: `openspec/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/tasks.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/verification.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/check-aspen-phase1-counts.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/helper-script-syntax.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/openspec-validate-change.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-change-tree.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-list.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-validate-specs.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-review.diff`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/prepare-aspen-phase1-worktree.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.sh`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-coordination-spotcheck.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-default-run.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/task-checklist.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-dylint-parse.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-dylint-parse.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-script-check.txt`

## Task Coverage

- [x] Update `docs/downstream-rollout.md` so Aspen phase-1 reconstruction documents the sandbox parent and sibling symlink setup that the working proof requires
  - Evidence: `docs/downstream-rollout.md` (`Aspen phase-1 reconstruction`), `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.txt`
- [x] Update consistency checks to assert against the final documented reconstruction path, not the older plain-worktree path
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.txt`
- [x] Re-run archived verification transcripts so command lines reference `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/...`
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/helper-script-syntax.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.txt`
- [x] Capture a new reconstruction transcript from the archived helper scripts that matches the final documented steps exactly
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-default-run.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-coordination-spotcheck.txt`
- [x] Add an explicit archived task checklist artifact mapping every checked task to its evidence
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/task-checklist.txt`

## Validation Notes

- The final public Aspen reconstruction path now includes the sandbox parent and
  sibling symlinks for `aspen-dns` and `aspen-wasm-plugin`, which the working
  proof actually requires.
- The archived reconstruction transcript now uses the archived helper-script
  path `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/...`.
- The archived task checklist artifact keeps the checkbox list and evidence map
  together so review does not need to infer task coverage indirectly.

## Archive Final State

- Change archived under `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/`.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-change-tree.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-list.txt`
- Documentation specs validate cleanly after sync.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-validate-specs.txt`
- Synced documentation-spec and implementation changes are captured in a reviewable diff.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-review.diff`

## Verification Commands

- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/helper-script-syntax.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash -n templates/tigerstyle-check.sh'; bash -n templates/tigerstyle-check.sh; echo '$ bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.sh'; bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.sh; echo '$ python3 -m py_compile openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/*.py'; python3 -m py_compile openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/*.py; echo 'ok'; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/helper-script-syntax.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-script-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash templates/tigerstyle-check.sh --help'; bash templates/tigerstyle-check.sh --help; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-script-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-dylint-parse.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-dylint-parse.py'; python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-dylint-parse.py; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/template-dylint-parse.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.py'; python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.py; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/archive-consistency-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.txt`
  - Command: `cd /path/to/git/tigerstyle && bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.sh > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-default-run.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-coordination-spotcheck.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/task-checklist.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ sed -n "1,200p" openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/tasks.md'; sed -n '1,200p' openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/tasks.md; echo; echo 'Evidence map:'; echo '- Phase 1 task 1 -> docs/downstream-rollout.md (Aspen phase-1 reconstruction), evidence/archive-consistency-check.txt'; echo '- Phase 1 task 2 -> evidence/archive-consistency-check.py, evidence/archive-consistency-check.txt'; echo '- Phase 2 task 1 -> evidence/*.py, evidence/reconstruct-aspen-phase1.sh, evidence/helper-script-syntax.txt'; echo '- Phase 2 task 2 -> evidence/reconstruct-aspen-phase1.txt, evidence/reconstruct-default-run.txt, evidence/reconstruct-coordination-spotcheck.txt'; echo '- Phase 2 task 3 -> verification.md plus archive-path command lines in evidence/helper-script-syntax.txt, evidence/archive-consistency-check.txt, evidence/reconstruct-aspen-phase1.txt'; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/task-checklist.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-validate-specs.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate --specs --strict --no-interactive > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-validate-specs.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-list.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec list > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-openspec-list.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-change-tree.txt`
  - Command: `cd /path/to/git/tigerstyle && find openspec/changes -maxdepth 2 -type d | sort > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-change-tree.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-review.diff`
  - Command: `cd /path/to/git/tigerstyle && git diff HEAD -- AGENTS.md .agent/napkin.md docs/downstream-rollout.md openspec/specs/documentation/spec.md openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-archive-alignment/evidence/post-archive-review.diff`
