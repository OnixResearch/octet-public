# Verification: Prove The Exact Aspen Replay Path We Document

## Implementation Evidence

- Changed file: `AGENTS.md`
- Changed file: `.agent/napkin.md`
- Changed file: `docs/downstream-rollout.md`
- Changed file: `openspec/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/tasks.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/verification.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/check-aspen-phase1-counts.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/helper-script-syntax.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-change-tree.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-list.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-validate-specs.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-review.diff`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/prepare-aspen-phase1-worktree.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-default-run.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-coordination-spotcheck.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/task-checklist.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-dylint-parse.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-dylint-parse.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-script-check.txt`

## Task Coverage

- [x] Update `docs/downstream-rollout.md` so the exact Aspen phase-1 replay path is the archived helper command, with the sandbox/symlink layout described as what the helper performs
  - Evidence: `docs/downstream-rollout.md` (`Exact replay path from this repo`), `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt`
- [x] Tighten the saved count checker so it asserts the `aspen-hlc: 0 starter findings` claim explicitly
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/check-aspen-phase1-counts.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt`
- [x] Update consistency checks so they verify the final helper-driven replay path and the stricter count checker behavior
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.txt`
- [x] Re-run the archived helper-driven replay and save a transcript whose top-level command matches the final documented replay path exactly
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-default-run.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-coordination-spotcheck.txt`
- [x] Refresh archived verification notes and task-checklist evidence so the checked Phase 2 task is provably backed by the new transcript
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/verification.md`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/task-checklist.txt`

## Validation Notes

- The public Aspen replay section now points at the exact archived helper
  command that generated the saved transcript.
- The saved transcript begins with that same archive-local helper command before
  showing the helper's internal sandbox, symlink, copy, and replay steps.
- The stricter count checker now asserts that `aspen-hlc` was checked and that
  it emitted no warning summary line before printing `aspen-hlc: 0 starter
  findings`.

## Archive Final State

- Change archived under `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/`.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-change-tree.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-list.txt`
- Documentation specs validate cleanly after sync.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-validate-specs.txt`
- Synced documentation-spec and implementation changes are captured in a reviewable diff.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-review.diff`

## Verification Commands

- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/helper-script-syntax.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash -n templates/tigerstyle-check.sh'; bash -n templates/tigerstyle-check.sh; echo '$ bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh'; bash -n openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh; echo '$ python3 -m py_compile openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/*.py'; python3 -m py_compile openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/*.py; echo 'ok'; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/helper-script-syntax.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-script-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash templates/tigerstyle-check.sh --help'; bash templates/tigerstyle-check.sh --help; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-script-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-dylint-parse.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-dylint-parse.py'; python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-dylint-parse.py; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/template-dylint-parse.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.py'; python3 openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.py; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/archive-consistency-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt 2>&1'; bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-default-run.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-coordination-spotcheck.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/task-checklist.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ sed -n "1,200p" openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/tasks.md'; sed -n '1,200p' openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/tasks.md; echo; echo 'Evidence map:'; echo '- Phase 1 task 1 -> docs/downstream-rollout.md (Exact replay path from this repo), evidence/archive-consistency-check.txt, evidence/reconstruct-aspen-phase1.txt'; echo '- Phase 1 task 2 -> evidence/check-aspen-phase1-counts.py, evidence/reconstruct-aspen-phase1.txt'; echo '- Phase 1 task 3 -> evidence/archive-consistency-check.py, evidence/archive-consistency-check.txt'; echo '- Phase 2 task 1 -> evidence/reconstruct-aspen-phase1.txt'; echo '- Phase 2 task 2 -> verification.md, evidence/task-checklist.txt, evidence/reconstruct-aspen-phase1.txt'; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/task-checklist.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-validate-specs.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate --specs --strict --no-interactive > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-validate-specs.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-list.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec list > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-openspec-list.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-change-tree.txt`
  - Command: `cd /path/to/git/tigerstyle && find openspec/changes -maxdepth 2 -type d | sort > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-change-tree.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-review.diff`
  - Command: `cd /path/to/git/tigerstyle && git diff HEAD -- AGENTS.md .agent/napkin.md docs/downstream-rollout.md openspec/specs/documentation/spec.md openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/post-archive-review.diff`
