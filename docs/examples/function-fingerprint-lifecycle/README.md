# Function fingerprint lifecycle example

This directory contains a docs-backed smoke artifact for the function fingerprint
operator workflow.

- `reviewed.rs` is the final repaired fixture after an accepted source change.
- `lifecycle-transcript.md` records the exact `--update`, `--check`, `--diff`,
  repair, and final-check lifecycle with observed human-output summaries.

Use it as a compact reference for what a clean gate, drift, accepted rewrite, and
repaired gate look like in human mode.
