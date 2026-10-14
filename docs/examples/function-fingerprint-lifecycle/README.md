# Function fingerprint lifecycle example

This directory contains a documentation smoke artifact for the function-fingerprint operator workflow.

- `reviewed.rs` is the final repaired fixture after an accepted source change.
- `lifecycle-transcript.md` records the exact `--update`, `--check`, `--diff`,
  repair, and final-check lifecycle with observed human-output summaries.

Use it as a compact reference for these human-mode results:

- clean gate
- drift
- accepted rewrite
- repaired gate
