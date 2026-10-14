# Prove The Exact Aspen Replay Path We Document

## Why

`aspen-feedback-hardening-archive-alignment` fixed the sandbox/symlink story and
archive-path transcripts, but the final done review still found one mismatch:
we saved a successful replay from an archived helper script without making the
published reconstruction section explicitly point at that helper as the exact
replay path.

That left one task overclaimed and one proof gap open. The same review also
caught that the saved count checker never asserted the `aspen-hlc: 0 starter
findings` claim even though the docs and verification summary stated it.

## What Changes

- revise the Aspen reconstruction section to document the exact archived helper
  replay command instead of implying a different literal path was run
- strengthen the saved count checker so it asserts the `aspen-hlc` zero-findings
  claim explicitly
- rerun the archived replay and count evidence from the exact helper path the
  docs and verification now reference

## Capabilities

### Modified Capabilities

- `audit-ready-rollout-evidence`: the published replay command, the saved helper
  script, and the archived transcript now describe the same path exactly

## Impact

- **Files**: rollout docs, documentation spec, archived helper scripts,
  archived transcripts, archived verification notes
- **APIs**: no Rust API changes
- **Dependencies**: no new dependencies
- **Testing**: rerun the archived helper-driven replay and the stricter count
  checker, then save the refreshed outputs
