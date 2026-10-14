## Contract

- [x] [serial] Define API-stability rail command/options, checked package scope, and release-readiness boundary. r[octet.api_stability.semver_rail]
- [x] [serial] Define accepted baseline sources and fail-closed baseline diagnostics. r[octet.api_stability.baseline]
- [x] [serial] Define API-stability receipt fields, provenance, and non-claims. r[octet.api_stability.receipt]

## Implementation

- [x] [serial] Add pure baseline/scope validation and API-stability receipt DTOs. r[octet.api_stability.baseline]
- [x] [serial] Add runner shell integration for `cargo-semver-checks` with package, feature, target, and baseline inputs. r[octet.api_stability.semver_rail]
- [x] [serial] Add release/operator docs for when to run API-stability evidence. r[octet.api_stability.receipt]
- [x] [serial] Register API-stability checker and receipt families in provenance/artifact registries. r[octet.api_stability.receipt]

## Verification

- [x] [serial] Add positive compatible public API fixture. r[octet.api_stability.semver_rail]
- [x] [serial] Add negative removed-public-item and missing-baseline fixtures. r[octet.api_stability.baseline]
- [x] [serial] Add receipt tests for feature set, target triple, baseline identity, and non-claims. r[octet.api_stability.receipt]
- [x] [serial] Run targeted runner tests, docs checks, and Cairn validation/gates. r[octet.api_stability.receipt]
