## Why

Generated policy artifact consumers now validate individual proof-bundle shape, but the artifact directory can still be edited piecemeal after export. A downstream check that reads `tigerstyle-policies.json` plus sibling generated artifacts should be able to reject a locally modified, missing, or mismatched artifact before trusting policy or proof metadata.

## What Changes

- Export a first-class generated artifact integrity manifest for every policy artifact directory.
- Bind each generated artifact filename to its BLAKE3 digest and byte length.
- Validate that manifest before artifact-backed checks consume generated policy artifacts.
- Keep validation local and deterministic; no signing keys, network lookups, or credential dependencies are introduced in this slice.

## Capabilities

### Modified Capabilities
- `standards-codification`: generated artifact consumption becomes manifest-backed and rejects drift across the artifact bundle.

## Impact

- **Files**: `tigerstyle-standards` artifact export/check paths, Nickel contracts/fixtures, Nix artifact assertions, OpenSpec specs.
- **APIs**: adds `tigerstyle-artifact-manifest.json` to generated artifact directories.
- **Dependencies**: reuses existing BLAKE3 support.
- **Testing**: Rust CLI tests cover export, tamper rejection, and complete-bundle acceptance; Nickel/Nix checks cover fixture/export behavior.
