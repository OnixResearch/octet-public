## Contract

- [x] [serial] Define the external schema contract library, pinned schema metadata, generator metadata, and indexed entry point. r[octet.evidence_contracts.external_schema_library]
- [x] [serial] Define freshness and registry requirements for generated external schema contracts. r[octet.evidence_contracts.external_schema_freshness]
- [x] [serial] Define boundaries that keep external schema validation separate from Octet receipt compatibility and proof claims. r[octet.evidence_contracts.external_schema_boundaries]

## Implementation

- [x] [serial] Add a generator path that converts checked-in external JSON schemas into deterministic Nickel contracts and records generator capability metadata. r[octet.evidence_contracts.external_schema_library]
- [x] [serial] Add the indexed generated contract bundle and BLAKE3-backed manifest for selected external evidence schemas. r[octet.evidence_contracts.external_schema_library]
- [x] [serial] Register the generated external schema contract family in `standards/policies/registry.ncl` with generation and validation commands. r[octet.evidence_contracts.external_schema_freshness]
- [x] [serial] Wire external evidence import validation to reject malformed inputs before Octet summarizes them. r[octet.evidence_contracts.external_schema_boundaries]
- [x] [serial] Document the generator dialect support, predicate fallback caveats, and input-shape-only non-claims. r[octet.evidence_contracts.external_schema_boundaries]

## Verification

- [x] [serial] Add positive external evidence fixtures that pass generated Nickel contract validation. r[octet.evidence_contracts.external_schema_library]
- [x] [serial] Add negative fixtures for missing required fields, wrong field types, unsupported schema version, unsupported dialect, and malformed report shape. r[octet.evidence_contracts.external_schema_boundaries]
- [x] [serial] Add freshness tests proving stale generated contracts or stale manifests fail with artifact-identifying diagnostics. r[octet.evidence_contracts.external_schema_freshness]
- [x] [serial] Run targeted generator, policy registry, external import, and Cairn validation/gate checks. r[octet.evidence_contracts.external_schema_freshness]
