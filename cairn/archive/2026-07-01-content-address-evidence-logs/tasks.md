## Contract

- [x] [serial] Specify inline summary and content-addressed raw log retention rules. r[octet.evidence_log_storage.retention]
- [x] [serial] Specify BLAKE3 manifest fields for bulky evidence artifacts. r[octet.evidence_log_storage.manifest]
- [x] [serial] Specify fail-closed verification for missing or tampered externalized evidence. r[octet.evidence_log_storage.fail_closed]

## Implementation

- [x] [serial] Add a manifest format and helper for content-addressed evidence logs. r[octet.evidence_log_storage.manifest]
- [x] [serial] Convert selected bulky archive logs to summary-plus-manifest form without changing scoped claims. r[octet.evidence_log_storage.retention]
- [x] [serial] Wire evidence log verification into the relevant release or lifecycle validation rail. r[octet.evidence_log_storage.fail_closed]

## Verification

- [x] [serial] Add positive tests for a present artifact with matching BLAKE3 digest, byte count, command, and checked scope. r[octet.evidence_log_storage.manifest]
- [x] [serial] Add negative tests for missing artifact, wrong digest, stale byte count, missing command, and missing checked scope. r[octet.evidence_log_storage.fail_closed]
- [x] [serial] Run evidence verification, docs/link checks for transformed archive references, and Cairn validation/gates. r[octet.evidence_log_storage.retention]
