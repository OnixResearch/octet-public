## Why

Tigerstyle needs a review-grade way to tie safety-critical Rust functions to exact reviewed or verified source bodies. Line numbers, file paths, and prose claims drift too easily: a protocol transition can move, a verified primitive can be edited after proof, and an OpenSpec artifact can no longer prove which implementation was reviewed.

Unison's content-addressed-code model suggests a pragmatic Rust adaptation: content-address selected functions, not the whole language. Function fingerprints should make reviewed logic changes explicit without turning every formatting change or comment edit into churn.

## What Changes

- **Function fingerprints**: Add a standards policy for annotating selected Rust functions with BLAKE3-style fingerprints that are checked against normalized function tokens.
- **CLI support**: Add a `cargo tigerstyle fingerprint` subcommand family for checking, writing, updating, and reporting function fingerprints.
- **Attribute/comment contract**: Define a first-class `#[fingerprint("b3:...")]` attribute path for projects that opt into macro support, plus a comment-marker fallback for stable/library consumers.
- **Review metadata**: Require reviewed fingerprints to carry explicit review class metadata for security, protocol transition, verified primitive, safety arithmetic, or documented example use cases.
- **Hash stability boundaries**: Specify normalization rules and macro-call caveats so fingerprints are review gates, not accidental formatting gates.

## Capabilities

### New Capabilities
- `function-fingerprints`: Selected functions can be tied to stable content hashes and checked in local/CI workflows.
- `fingerprint-cli`: Consumers can generate, check, update, and diff fingerprints with explicit commands.

### Modified Capabilities
- `standards-codification`: Adds content-addressed review evidence as an advisory-to-hard policy family for critical functions.
- `consumer-integration`: Adds a command surface that produces deterministic diagnostics and update behavior for function fingerprints.

## Impact

- **Files**: New docs/policy entries, CLI parser/runner code, hash-normalization code, tests/UI fixtures, and templates/examples.
- **APIs**: Adds `cargo tigerstyle fingerprint --check|--write|--update|--diff` and optional output fields for fingerprint diagnostics.
- **Dependencies**: May add `blake3` and a Rust token parsing/formatting dependency if existing parser support is insufficient.
- **Testing**: Verify stable hashes across formatting/comment changes, mismatches on semantic token changes, macro-call warnings, stable fallback markers, and CI-style check failures.

## Non-Goals

- Do not content-address every Rust definition by default.
- Do not claim fingerprints prove semantic correctness.
- Do not hash absolute paths, line numbers, timestamps, local usernames, secrets, or full environments.
- Do not require a proc-macro dependency for stable/published consumers that only want comment-marker validation.
