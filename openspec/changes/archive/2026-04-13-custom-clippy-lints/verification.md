# Verification: Custom Clippy Lints Archive

This verification file was added after archive to close review gaps around
spec sync, Aspen rollout evidence, and saved validation transcripts.

## Implementation Evidence

- Changed file: `README.md`
- Changed file: `openspec/specs/purity/spec.md`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/proposal.md`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/design.md`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/specs/purity/spec.md`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/verification.md`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-default-pilot-rerun.txt`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-time-core-tests.txt`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-time-core-clippy.txt`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-validation-summary.txt`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/post-archive-openspec-list.txt`
- Changed file: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/spec-validate.txt`

## Task Coverage

- [x] Add tigerstyle invocation to Aspen's equivalent check path (`scripts/tigerstyle-check.sh`)
  - Evidence: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`
- [x] Configure Aspen `dylint.toml` thresholds and allowlists for known exceptions
  - Evidence: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`

## Review Gap Fixes

- `ambient_clock` is now described in the archived purity delta, the canonical
  purity spec, the archived proposal, the archived design, and `README.md`.
  - Evidence: `openspec/specs/purity/spec.md`, `openspec/changes/archive/2026-04-13-custom-clippy-lints/specs/purity/spec.md`, `openspec/changes/archive/2026-04-13-custom-clippy-lints/proposal.md`, `openspec/changes/archive/2026-04-13-custom-clippy-lints/design.md`, `README.md`
- Archived change status is preserved and there are no active Tiger Style
  changes left in this repo.
  - Evidence: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/post-archive-openspec-list.txt`
- Canonical Tiger Style specs validate after the retroactive sync fixes.
  - Evidence: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/spec-validate.txt`

## Validation Commands

### `scripts/tigerstyle-check.sh`

- Status: pass
- Artifact: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-default-pilot-rerun.txt`

### `cargo test -p aspen-time -p aspen-core --lib`

- Status: pass
- Artifact: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-time-core-tests.txt`

### `cargo clippy -p aspen-time -p aspen-core --lib -- -D warnings`

- Status: pass
- Artifact: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-time-core-clippy.txt`

### Validation task metadata

- Status: success
- Artifact: `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-validation-summary.txt`
