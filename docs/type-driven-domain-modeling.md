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

## Nominal-domain policy

`standards/nominal-domains/catalog.ncl` is the reviewed source of truth. Its
JSON export gives the lint driver deterministic runtime data.

Each policy row declares these items:

- the domain symbols and raw representation
- the core source scopes
- explicit compatibility scopes
- the wrapper and constructor contract

`primitive_domain_alias` finds raw type aliases. `raw_domain_value` checks
registered fields and API values. `newtype_invariant_bypass` checks refined
wrapper escape paths.

Use this boundary flow:

```text
wire DTO -> checked adapter -> typed core -> wire projection
```

Wire, FFI, protocol, generated, and vendored scopes can stay structural. Each
exception must be explicit. A typed adapter must reject malformed values before
they enter the core.

## Rollout modes

Omit `nominal_domain_mode` to keep the three lints at their default levels.
Set it to `inventory` for non-blocking exact and suffix findings. Set it to
`warn` for exact warnings only. Set it to `deny` for exact errors only.

A passing check proves configured source-shape conformance. It does not prove
authority, secrecy, runtime correctness, wire safety, or release eligibility.
