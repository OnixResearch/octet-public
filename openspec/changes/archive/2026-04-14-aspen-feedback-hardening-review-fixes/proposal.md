# Fix Review Gaps In Aspen Feedback Hardening

## Why

The archived `aspen-feedback-hardening` change shipped the right rollout kit,
but its done review found three gaps in the proof:

- the documented minimum assumptions did not match the published template
- the archive did not include a real reconstruction transcript using the new
  docs/templates
- verification artifacts elided the helper scripts, which made independent
  audit harder than it should be

Those are not new product goals. They are missing implementation and evidence
for the rollout kit we already claimed was complete.

## What Changes

- align `docs/downstream-rollout.md` with the template's real runtime
  assumptions
- remove any avoidable template dependency that the doc does not need to carry
- capture an end-to-end Aspen phase-1 reconstruction transcript using the new
  templates in a disposable worktree
- store full helper scripts and full verification commands in evidence instead
  of elided `python - <<'PY' ...` summaries

## Capabilities

### Modified Capabilities

- `consumer-integration-kit`: published assumptions now match the actual
  template/runtime dependencies
- `high-noise-lint-promotion-playbook`: unchanged in scope, but now backed by
  audit-ready reconstruction evidence

### New Capabilities

- `audit-ready-rollout-evidence`: reviewers can inspect the exact helper
  scripts and a real reconstruction transcript instead of relying on summarized
  success messages

## Impact

- **Files**: rollout docs, template script, archived verification artifacts,
  documentation spec
- **APIs**: no Rust API changes
- **Dependencies**: no new runtime dependencies; remove explicit `realpath`
  reliance from template
- **Testing**: rerun Aspen phase-1 reconstruction from the new template in a
  disposable worktree and save the full transcript
