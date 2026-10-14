# Prove The Transcript Capture Method We Claim

## Why

`aspen-feedback-hardening-exact-replay` fixed the helper/doc path mismatch, but
its done review found one more proof gap: the saved transcript starts with a
`$ bash ...` line that the replay helper itself never prints. That means the
archive still does not prove how the transcript was captured, and Phase 2 stays
overclaimed.

## What Changes

- document one exact archive-local command that both runs the Aspen replay and
  writes the saved transcript
- store the transcript-capture helper as a reviewable file beside the replay
  helper it wraps
- strengthen consistency checks so they verify the transcript preamble comes
  from the documented capture helper, not from an undocumented outer wrapper

## Capabilities

### Modified Capabilities

- `audit-ready-rollout-evidence`: the documented transcript-regeneration
  command, the saved capture helper, and the saved transcript now describe one
  identical archive-local path

## Impact

- **Files**: rollout docs, documentation spec, archived evidence scripts,
  archived transcript, archived verification notes
- **APIs**: no Rust API changes
- **Dependencies**: no new dependencies
- **Testing**: rerun the archive-local capture helper, consistency checks, and
  OpenSpec validation after archiving
