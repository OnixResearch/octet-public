# External schema contracts

External evidence rails may import third-party JSON shapes. Octet validates selected input shapes with checked-in generated Nickel contracts before treating reports as accepted input.

Current bundle:

- `standards/external-schemas/index.ncl`
- `standards/external-schemas/contracts/generated.ncl`
- `standards/external-schemas/manifest.json`
- pinned schemas under `standards/external-schemas/pinned/`
- positive and negative fixtures under `standards/external-schemas/fixtures/`

The generator records schema source path, JSON Schema dialect, tool family, generator identity, generator mode, unsupported keyword caveats, byte length, and BLAKE3 digest. Validation is offline only: no SchemaStore, GitHub, tool docs, or remote schema fetches are used during freshness checks.

Passing a generated contract means Octet accepts the report shape. It does not prove that the external tool result is complete, correct, vulnerability-free, or formally verified.
