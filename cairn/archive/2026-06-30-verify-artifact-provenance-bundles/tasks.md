## Contract

- [x] [serial] Specify the artifact bundle verifier command, accepted input options, deterministic output shape, and non-mutating behavior. r[octet.artifact_bundle_verification.command]
- [x] [serial] Specify complete-bundle file-set validation and provenance coverage for every non-provenance artifact. r[octet.artifact_bundle_verification.complete_bundle]
- [x] [serial] Specify fail-closed diagnostics for malformed, partial, stale, or inconsistent bundles. r[octet.artifact_bundle_verification.fail_closed]

## Implementation

- [x] [serial] Add a pure bundle verification core over expected files, artifact hashes, provenance records, checker IDs, and evidence tiers. r[octet.artifact_bundle_verification.complete_bundle]
- [x] [serial] Add the CLI shell that reads `--artifact-dir`, computes BLAKE3 hashes, parses `provenance.jsonl`, and renders deterministic reports without mutating the bundle. r[octet.artifact_bundle_verification.command]
- [x] [serial] Update release/operator docs to use the verifier before treating artifact directories as release evidence. r[octet.artifact_bundle_verification.command]

## Verification

- [x] [serial] Add positive tests for complete human, JSONL, and SARIF bundles. r[octet.artifact_bundle_verification.complete_bundle]
- [x] [serial] Add negative tests for missing files, stale extras, malformed JSONL, hash mismatch, unknown checker ID, invalid tier, and missing human action. r[octet.artifact_bundle_verification.fail_closed]
- [x] [serial] Run targeted cargo-octet tests, consumer artifact integration tests, policy validation, and Cairn validation/gates. r[octet.artifact_bundle_verification.command]
