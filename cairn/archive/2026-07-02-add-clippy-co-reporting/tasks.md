## Contract

- [x] [serial] Define Clippy run/import modes and external-lint sidecar shape. r[octet.external_lint.clippy_co_reporting]
- [x] [serial] Define source severity versus Octet gate outcome rendering. r[octet.external_lint.severity_mapping]
- [x] [serial] Define Clippy checker provenance and optional-artifact verification behavior. r[octet.external_lint.provenance]

## Implementation

- [x] [serial] Add pure parser/normalizer for Clippy JSON messages. r[octet.external_lint.clippy_co_reporting]
- [x] [serial] Add runner shell support for Clippy run mode and report import mode. r[octet.external_lint.clippy_co_reporting]
- [x] [serial] Add external-lint receipt/provenance DTOs and artifact verifier support. r[octet.external_lint.provenance]
- [x] [serial] Document Clippy as complementary hygiene evidence, not an Octet lint replacement. r[octet.external_lint.severity_mapping]

## Verification

- [x] [serial] Add positive Clippy JSON import fixture and run-mode smoke. r[octet.external_lint.clippy_co_reporting]
- [x] [serial] Add negative malformed JSON and missing-field fixtures. r[octet.external_lint.severity_mapping]
- [x] [serial] Add artifact verification coverage for optional Clippy receipt provenance. r[octet.external_lint.provenance]
- [x] [serial] Run targeted runner tests, artifact verification tests, and Cairn validation/gates. r[octet.external_lint.provenance]
