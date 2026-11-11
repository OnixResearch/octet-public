# Align Aspen Reconstruction Docs And Archived Evidence

## Why

The second Aspen hardening follow-up fixed most review gaps, but one mismatch
remains: the published reconstruction steps still show a plain Aspen worktree,
while the only successful proof uses a sandbox parent plus sibling symlinks for
`aspen-dns` and `aspen-wasm-plugin`.

That means the docs are still incomplete relative to the working evidence, and
some archived transcripts still refer to the pre-archive active-change path
instead of the archived location.

## What Changes

- update the published Aspen reconstruction steps to document the sandbox parent
  and sibling symlink setup that the working proof requires
- re-run reconstruction and verification artifacts from the archived location so
  transcript command lines reference the archive, not the removed active change
- add an explicit task checklist artifact so reviewers can verify every checked
  task from the archived evidence set alone

## Capabilities

### Modified Capabilities

- `audit-ready-rollout-evidence`: the documented reconstruction path now matches
  the successful transcript exactly, and archived artifacts are self-consistent

## Impact

- **Files**: rollout guide, documentation spec, archived evidence transcripts,
  archived verification notes
- **APIs**: no Rust API changes
- **Dependencies**: no new runtime dependencies
- **Testing**: re-run reconstruction from archived helper scripts and validate
  that the transcript matches the final documented steps
