# External schema contracts

External evidence rails may import third-party JSON shapes. Octet validates selected input shapes with checked-in generated Nickel contracts before treating reports as accepted input.

Current bundle:

- `standards/external-schemas/index.ncl`
- `standards/external-schemas/contracts/generated.ncl`
- `standards/external-schemas/manifest.json`
- pinned schemas under `standards/external-schemas/pinned/`
- positive and negative fixtures under `standards/external-schemas/fixtures/`

The generator derives each Nickel contract from its pinned schema and records schema source path, JSON Schema dialect, tool family, generator identity, generator mode, unsupported keywords (empty for every generated contract, because unsupported keywords fail generation), byte length, and BLAKE3 digest. Validation is offline only: no SchemaStore, GitHub, tool docs, or remote schema fetches are used during freshness checks.

Translation rules:

- The dialect is read from the pinned schema's `$schema`. Generation fails closed unless it is `https://json-schema.org/draft/2020-12/schema`.
- A property listed in `required` becomes a mandatory field with no default. Every other property becomes an `optional` field, so the contract never invents values.
- `additionalProperties` that is absent or `true` produces an open record (`..`). `false` produces a closed record.
- Supported validation keywords are `type` (`object`, `array`, `string`, `number`, `integer`, `boolean`, `null`), `properties`, `required`, `additionalProperties` (boolean only), `items`, `const` (scalar only), `minLength`, `minimum`, and `maximum`. `$id`, `$comment`, `title`, and `description` are annotations and are ignored.
- Any other keyword fails generation with a diagnostic naming the keyword and its schema location, so a passing contract never silently drops a constraint.

Refresh the checked-in bundle with `nix develop -c cargo test -p cargo-octet --lib external_schema_contracts::tests::regenerate_external_schema_contracts -- --ignored`, then run `nix develop -c cargo test -p cargo-octet --lib external_schema_contracts` to check freshness and fixtures.

Passing a generated contract means Octet accepts the report shape. It does not prove that the external tool result is complete, correct, vulnerability-free, or formally verified.
