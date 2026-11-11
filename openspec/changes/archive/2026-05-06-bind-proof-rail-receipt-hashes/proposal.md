# bind-proof-rail-receipt-hashes Proposal

## Why

Verified proof rails now execute local commands and check pass signals, but a saved or generated receipt is not yet cryptographically bound to the exact source text and command that produced it. A stale receipt can carry the expected pass signal without proving which command and source version it covered.

## What Changes

- Add deterministic receipt-binding requirements to verified proof rails.
- Generate BLAKE3 hash lines for the proof command and each matched source file when `--execute-proof-rails` runs.
- Validate proof receipts by recomputing the configured command/source hashes from the checked source set and rejecting receipts with missing or mismatched binding lines.
- Keep proof execution local and deterministic; no network or credential dependency is introduced.

## Impact

- **Rust:** `tigerstyle-standards` proof receipt generation and proof checking.
- **Nickel/artifacts:** proof-rail metadata gains receipt-binding fields.
- **Fixtures:** verification fixtures gain hash-bound receipt coverage.
- **Verification:** focused CLI tests, policy artifact assertions, standards fixtures, OpenSpec validation, and docs examples.
