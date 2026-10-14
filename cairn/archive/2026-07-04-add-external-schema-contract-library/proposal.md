## Why

Octet already treats Nickel as the source for human-authored policy and as a generated contract surface for Rust-owned receipts. Optional external evidence rails still need a consistent way to validate third-party JSON shapes before Octet imports or summarizes them. The Nickel ecosystem has two useful patterns for this: `json-schema-to-nickel` converts JSON Schema into Nickel contracts, and `nickel-schemastore` packages generated contracts as a reusable indexed library.

Adapting those patterns would let Octet validate selected external evidence inputs with checked-in, deterministic Nickel contracts while preserving Octet's current rule that Rust DTOs own Octet-emitted receipt compatibility.

## What Changes

- Add a checked-in generated Nickel contract library for selected external evidence schemas that Octet already consumes or plans to consume.
- Pin every source schema locally and record schema dialect, generator identity, BLAKE3 source digest, generated artifact digest, and generation command.
- Register the generated contract family in `standards/policies/registry.ncl` with positive and negative fixtures.
- Make import validation fail closed on stale generated contracts, missing required fields, wrong field types, unsupported schema versions, or unsupported schema dialects.
- Document generated external contracts as input-shape validation, not as proof that the external tool result is correct.

## Non-Goals

- Do not replace Rust DTO ownership for Octet-emitted receipts.
- Do not require consumers to evaluate Nickel at `cargo octet check` runtime.
- Do not fetch SchemaStore, GitHub, or tool schemas from the network during validation.
- Do not claim full JSON Schema semantic coverage beyond the generator's supported dialect and documented fallbacks.

## Impact

- **Files**: external schema contract generation code, `standards/` generated Nickel contract bundle, policy registry, fixtures, docs, and Cairn specs.
- **Behavior**: selected external evidence imports gain deterministic contract validation and freshness checks before evidence is trusted as Octet input.
- **Testing**: generator freshness tests, positive/negative schema fixtures, stale artifact rejection, unsupported dialect rejection, and Cairn validation/gates.
