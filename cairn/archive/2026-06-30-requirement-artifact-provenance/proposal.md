## Why

external evidence tooling keeps provenance from source formulas and requirements through properties, proof artifacts, and emitted code. Octet already has many provenance ingredients: OpenSpec/Cairn requirement IDs, standards policy IDs, lint names, SARIF, sidecars, transcripts, fingerprints, and artifact manifests. They are not yet presented as one requirement-to-artifact chain.

Octet should make provenance first-class: when an artifact claims a check, reviewers should be able to trace the claim back to the requirement, policy, lint, or transcript assertion that authorized it.

## What Changes

- Define a provenance record shape for evidence artifacts that includes source requirement or policy ID, checker identity, evidence tier, artifact path or URI, BLAKE3 content hash, and boundary/scope.
- Add provenance fields to new or versioned sidecar/evidence summaries where Octet claims policy or requirement coverage.
- Document the source-to-artifact chain and how release operators inspect it.
- Add validation fixtures proving that missing source IDs or mismatched hashes fail closed.

## Impact

- **Files**: evidence DTOs under `cargo-octet/src/` or `crates/octet-standards/src/`, `standards/policies/registry.ncl`, `octet-artifact-manifest.json` freshness coverage, release/operator docs, and example transcripts.
- **Testing**: positive provenance fixture with requirement ID and BLAKE3 hash; negative fixtures for missing source ID, unknown checker ID, and hash mismatch.
