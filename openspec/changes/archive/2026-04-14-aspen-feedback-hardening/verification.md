# Verification: Harden Tigerstyle From Aspen Rollout Findings

## Implementation Evidence

- Changed file: `AGENTS.md`
- Changed file: `.agent/napkin.md`
- Changed file: `README.md`
- Changed file: `docs/downstream-rollout.md`
- Changed file: `templates/tigerstyle-check.sh`
- Changed file: `templates/dylint.toml`
- Changed file: `openspec/specs/documentation/spec.md`
- Changed file: `openspec/specs/purity/spec.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/tasks.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/verification.md`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-script-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-dylint-parse.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/archive-consistency-check.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/aspen-head-drift-note.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/openspec-validate-change.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-validate-specs.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-list.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-change-tree.txt`
- Changed file: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/spec-sync.diff`

## Task Coverage

- [x] Document named starter rollout profiles, their intended first-use scope, and the promotion checklist for adding more crates/packages
  - Evidence: `docs/downstream-rollout.md` (`Starter rollout profiles`, `Promotion checklist for adding crates or packages`)
- [x] Add a reusable downstream check-path recipe/template for nix-managed or otherwise nonstandard Dylint toolchains
  - Evidence: `templates/tigerstyle-check.sh`, `docs/downstream-rollout.md` (`Check-path recipe for nonstandard toolchains`), `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-script-check.txt`
- [x] Add a sample `dylint.toml` template with commented threshold, allowlist, and rationale sections for consumer-owned policy
  - Evidence: `templates/dylint.toml`, `docs/downstream-rollout.md` (`Consumer-owned dylint.toml`), `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-dylint-parse.txt`
- [x] Add `ambient_clock` boundary-pattern guidance for timeout/deadline helpers, elapsed-time measurement, and shell-owned wait loops
  - Evidence: `docs/downstream-rollout.md` (`ambient_clock boundary playbook`)
- [x] Add a high-noise lint promotion playbook covering `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, and `unbounded_loop`
  - Evidence: `docs/downstream-rollout.md` (`High-noise lint promotion playbook`)
- [x] Document known classification patterns and anti-patterns, including when narrow item-scoped `allow`s are acceptable and when blanket suppression is not
  - Evidence: `docs/downstream-rollout.md` (`Inventory expectations by family`, `Narrow allow rules`)
- [x] Validate the new kit against Aspen's archived rollout evidence and update any gaps or contradictions it exposes
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/archive-consistency-check.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/aspen-head-drift-note.txt`, `docs/downstream-rollout.md` (`Reference state: use the Aspen archive, not Aspen HEAD`)
- [x] Capture evidence showing that a maintainer can reconstruct Aspen's phase-1 rollout path from the new `tigerstyle` guidance without relying on undocumented shell state
  - Evidence: `docs/downstream-rollout.md` (`Aspen phase-1 reconstruction`), `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/archive-consistency-check.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-script-check.txt`

## Validation Notes

Validation exposed one real contradiction: Aspen's current workspace script has
already moved past the archived phase-1 starter path by defaulting to
`aspen-coordination` and extra safety lints. The new kit resolves that by
pinning starter-path reconstruction to the archived Aspen rollout evidence
instead of Aspen HEAD.

## Archive Final State

- Change archived under `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/`.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-change-tree.txt`, `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-list.txt`
- Main documentation and purity specs validate cleanly after sync.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-validate-specs.txt`
- Synced spec deltas are captured in a reviewable diff.
  - Evidence: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/spec-sync.diff`

## Verification Commands

- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-script-check.txt`
  - Command: `cd /path/to/git/tigerstyle && { echo '$ bash -n templates/tigerstyle-check.sh'; bash -n templates/tigerstyle-check.sh; echo ok; echo; echo '$ bash templates/tigerstyle-check.sh --help'; bash templates/tigerstyle-check.sh --help; } > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-script-check.txt 2>&1`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/template-dylint-parse.txt`
  - Command: `cd /path/to/git/tigerstyle && python - <<'PY' ... tomllib.loads(Path('templates/dylint.toml').read_text()) ... PY`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/archive-consistency-check.txt`
  - Command: `cd /path/to/git/tigerstyle && python - <<'PY' ... assert docs/downstream-rollout.md matches archived Aspen phase-1 scope, counts, and wrapper facts ... PY`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/aspen-head-drift-note.txt`
  - Command: `cd /path/to/git/tigerstyle && sed -n '1,80p' /path/to/git/aspen/scripts/tigerstyle-check.sh > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/aspen-head-drift-note.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/openspec-validate-change.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate aspen-feedback-hardening --strict --no-interactive > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/openspec-validate-change.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-validate-specs.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec validate --specs --strict --no-interactive > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-validate-specs.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-list.txt`
  - Command: `cd /path/to/git/tigerstyle && openspec list > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-openspec-list.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-change-tree.txt`
  - Command: `cd /path/to/git/tigerstyle && find openspec/changes -maxdepth 2 -type d | sort > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/post-archive-change-tree.txt`
- Artifact: `openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/spec-sync.diff`
  - Command: `cd /path/to/git/tigerstyle && git diff HEAD -- openspec/specs/documentation/spec.md openspec/specs/purity/spec.md > openspec/changes/archive/2026-04-14-aspen-feedback-hardening/evidence/spec-sync.diff`
