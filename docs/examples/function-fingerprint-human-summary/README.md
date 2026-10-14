# Function fingerprint human summary example

This example documents the human-output path for `cargo octet fingerprint`.
It shows two reviewed functions, a one-function drift, the non-zero check result,
and the repair update that rewrites exactly one marker.

The executable regression for this directory asserts that the transcript stdout
stays synchronized with the CLI and that the repaired scratch fixture exactly
matches `reviewed.rs`.
