# Verification: Fix Review Gaps In Aspen Feedback Hardening

## Implementation Evidence

- Changed file: `AGENTS.md`
- Changed file: `.agent/napkin.md`
- Changed file: `docs/downstream-rollout.md`
- Changed file: `templates/tigerstyle-check.sh`
- Changed file: `openspec/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/specs/documentation/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/tasks.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/verification.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/helper-script-syntax.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-script-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/prepare-aspen-phase1-worktree.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/check-aspen-phase1-counts.py`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-default-run.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-coordination-spotcheck.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/openspec-validate-change.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-validate-specs.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-list.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-change-tree.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-review.diff`

## Task Coverage

- [x] Remove the template's avoidable `realpath` dependency and document the remaining required commands/environment assumptions accurately
  - Evidence: `templates/tigerstyle-check.sh`, `docs/downstream-rollout.md`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.txt`
- [x] Update the rollout guide so its minimum-assumption section matches the published template behavior exactly
  - Evidence: `docs/downstream-rollout.md` (`Minimum assumptions for the template`), `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-script-check.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.py`
- [x] Store the full helper scripts used for validation as reviewable evidence files
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/prepare-aspen-phase1-worktree.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/check-aspen-phase1-counts.py`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh`
- [x] Capture an end-to-end Aspen phase-1 reconstruction transcript using the new templates in a disposable worktree
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-default-run.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-coordination-spotcheck.txt`
- [x] Refresh verification notes so every evidence artifact points to a concrete command or stored helper script instead of elided here-docs
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/verification.md`

## Validation Notes

- The template no longer depends on `realpath`; it resolves `TIGERSTYLE_PATH`
  through `python3`.
- The reconstruction must pin Aspen to commit `909e5e9fa`. Aspen HEAD is too
  clean to reproduce the archived phase-1 warning counts.
- Aspen cannot be reconstructed from a naive temp worktree because sibling path
  dependencies (`aspen-dns`, `aspen-wasm-plugin`) and their references back to
  `../aspen` break resolution. The saved reconstruction script uses a sandbox
  parent with sibling symlinks and an `aspen/` worktree at the archived commit.
- The template's starter profile is now documented as a reference profile. The
  previous attempt to force per-lint selection through external flags was not a
  reliable Dylint integration path.

## Archive Final State

- Change archived under `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/`.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-change-tree.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-list.txt`
- Documentation specs validate cleanly after sync.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-validate-specs.txt`
- Synced documentation spec and implementation changes are captured in a reviewable diff.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-review.diff`

## Verification Commands

- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/helper-script-syntax.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash -n templates/tigerstyle-check.sh'; bash -n templates/tigerstyle-check.sh; echo '$ bash -n openspec/changes/aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh'; bash -n openspec/changes/aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh; echo '$ python3 -m py_compile openspec/changes/aspen-feedback-hardening-review-fixes/evidence/*.py'; python3 -m py_compile openspec/changes/aspen-feedback-hardening-review-fixes/evidence/*.py; echo 'ok'; } > openspec/changes/aspen-feedback-hardening-review-fixes/evidence/helper-script-syntax.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-script-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash templates/tigerstyle-check.sh --help'; bash templates/tigerstyle-check.sh --help; } > openspec/changes/aspen-feedback-hardening-review-fixes/evidence/template-script-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.py'; python3 openspec/changes/aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.py; } > openspec/changes/aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.txt 2>&1`
  - Helper script: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/template-dylint-parse.py`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ python3 openspec/changes/aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.py'; python3 openspec/changes/aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.py; } > openspec/changes/aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.txt 2>&1`
  - Helper script: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/archive-consistency-check.py`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/openspec-validate-change.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate aspen-feedback-hardening-review-fixes --strict --no-interactive > openspec/changes/aspen-feedback-hardening-review-fixes/evidence/openspec-validate-change.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.txt`
  - Command: `cd /path/to/git/tigerstyle && bash openspec/changes/aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh > openspec/changes/aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.txt 2>&1`
  - Helper scripts:
    - `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh`
    - `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/prepare-aspen-phase1-worktree.py`
    - `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/check-aspen-phase1-counts.py`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-default-run.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-coordination-spotcheck.txt`
  - Produced by: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/reconstruct-aspen-phase1.sh`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-validate-specs.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate --specs --strict --no-interactive > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-validate-specs.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-list.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec list > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-openspec-list.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-change-tree.txt`
  - Command: `cd /path/to/git/tigerstyle && find openspec/changes -maxdepth 2 -type d | sort > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-change-tree.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-review.diff`
  - Command: `cd /path/to/git/tigerstyle && git diff HEAD -- openspec/specs/documentation/spec.md docs/downstream-rollout.md templates/tigerstyle-check.sh > openspec/changes/archive/2026-04-14-aspen-feedback-hardening-review-fixes/evidence/post-archive-review.diff`
