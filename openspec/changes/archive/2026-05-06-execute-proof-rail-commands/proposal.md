## Why

Verified proof rails currently enforce committed receipt shape and pass-signal presence. That catches missing or failing receipts, but a stale checked-in receipt can still satisfy the gate without proving that the configured proof command was run for the checked source root.

## What Changes

- Add an opt-in command-execution mode for `tigerstyle-standards check` that runs relevant configured proof rail commands before evaluating proof receipts.
- Capture command stdout, stderr, exit status, and command identity as the in-memory proof receipt used by the checker.
- Keep normal artifact-backed receipt checks available for read-only downstream consumers that do not opt into command execution.
- Add fixture coverage proving executed proof commands pass for compliant verification fixtures and fail for noncompliant fixtures.

## Impact

- **Files**: `crates/tigerstyle-standards`, verification fixtures, `standards/policies/registry.ncl`, generated artifact assertions, docs.
- **APIs**: `tigerstyle-standards check` gains an explicit command-execution flag.
- **Testing**: Rust unit tests, policy validation, standards fixtures, docs examples, and OpenSpec validation.
