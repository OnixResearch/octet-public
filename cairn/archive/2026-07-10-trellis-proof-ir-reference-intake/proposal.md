## Why

Octet consumes Trellis as a verified-logic reference corpus, but imported Trellis artifacts must not automatically become Octet proof claims. A Trellis proof evidence IR gives Octet a stronger intake format than source-only artifacts while preserving the boundary between reference evidence and Octet-rerun formal proof receipts.

## What Changes

- Add a Trellis proof evidence IR intake profile for Octet evidence rails.
- Treat imported Trellis proof evidence as recorded-only reference input by default.
- Define promotion conditions for Octet-rerun proof receipts: matching source, verifier/toolchain identity, assumptions, property scope, command identity, and result status.
- Add positive and negative fixtures for reference intake and promotion rejection.

## Impact

Octet can use Trellis proof IR for traceable references without overclaiming downstream Rust correctness or Trellis verifier results it did not rerun.
