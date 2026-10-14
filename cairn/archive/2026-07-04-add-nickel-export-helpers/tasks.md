## Contract

- [x] [serial] Define the deterministic Nickel export helper interface and accepted output formats. r[octet.nickel_export_helpers.export]
- [x] [serial] Define BLAKE3-backed export manifest and freshness verification requirements. r[octet.nickel_export_helpers.manifest]
- [x] [serial] Define runtime, native-format, and secret-boundary constraints for Nickel-authored exports. r[octet.nickel_export_helpers.boundaries]

## Implementation

- [x] [serial] Add the Nix/flake helper or small Octet wrapper for deterministic Nickel export. r[octet.nickel_export_helpers.export]
- [x] [serial] Add manifest generation and verification for helper-produced artifacts. r[octet.nickel_export_helpers.manifest]
- [x] [serial] Register helper-produced config or receipt families in `standards/policies/registry.ncl` as `nickel-authored` where applicable. r[octet.nickel_export_helpers.manifest]
- [x] [serial] Migrate one existing Nickel-authored export path as the pilot without changing its emitted artifact contract. r[octet.nickel_export_helpers.export]
- [x] [serial] Document generation-time Nickel usage, runtime no-Nickel expectations, and cases that should remain tool-native config. r[octet.nickel_export_helpers.boundaries]

## Verification

- [x] [serial] Add positive fixtures showing stable JSON, TOML, YAML, and raw exports where supported. r[octet.nickel_export_helpers.export]
- [x] [serial] Add negative fixtures for missing source, failing contract, unsupported format, stale manifest, and stale output bytes. r[octet.nickel_export_helpers.manifest]
- [x] [serial] Add a secret-bearing fixture that is rejected or classified as a non-candidate before manifest inclusion. r[octet.nickel_export_helpers.boundaries]
- [x] [serial] Run targeted export helper, manifest, policy freshness, and Cairn validation/gate checks. r[octet.nickel_export_helpers.manifest]
