# Type-driven domain modeling

Octet uses small Rust domain types at selected core boundaries to prevent raw
string, path, numeric, and boolean combinations from reaching evidence logic
unchecked. The goal is boring algebraic modeling: newtypes for domain identities,
enums for mutually exclusive states, and refined constructors for non-empty,
canonical, sorted/deduplicated, path-normalized, and threshold/count values.

The first migrated runner seam validates finding DTOs before JSONL and SARIF
rendering while preserving existing wire fields. Lint names, group names, stable
IDs, semantic IDs, artifact locators, severity, and one-based source positions
are checked through adapters; output remains the same for valid findings.
Malformed identities, non-canonical paths, unsupported severities, and zero line
or column positions fail before rendering.

This is not Haskell cleverness. Octet should avoid lens-heavy APIs, monad
terminology, laziness assumptions, macro-heavy encodings, and type-level
machinery unless a later change ties them to a concrete bug-prevention payoff.
Runtime receipts and emitted evidence DTOs stay Rust-owned; domain types protect
internal cores and adapters without creating a second wire-format source of
truth.
