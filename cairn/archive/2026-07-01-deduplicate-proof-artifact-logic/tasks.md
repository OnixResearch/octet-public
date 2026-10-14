## Contract

- [x] [serial] Specify shared proof/artifact identity helpers and ownership. r[octet.proof_artifact_library.shared_identity]
- [x] [serial] Specify shared proof receipt binding and relevance decisions. r[octet.proof_artifact_library.proof_binding]
- [x] [serial] Specify equivalence and fail-closed validation across runner and standards paths. r[octet.proof_artifact_library.fail_closed_equivalence]

## Implementation

- [x] [serial] Extract BLAKE3 identity, canonical closure graph, and artifact manifest comparison logic into a shared pure module. r[octet.proof_artifact_library.shared_identity]
- [x] [serial] Route standards proof validation and cargo-octet prove paths through the shared binding/relevance logic. r[octet.proof_artifact_library.proof_binding]
- [x] [serial] Preserve command-specific I/O, rendering, and diagnostics in thin shell modules. r[octet.proof_artifact_library.fail_closed_equivalence]

## Verification

- [x] [serial] Add positive tests proving valid receipts and manifests pass through both command surfaces. r[octet.proof_artifact_library.shared_identity]
- [x] [serial] Add negative tests for stale hashes, missing bindings, unsupported schemas, malformed closure graphs, and missing provenance. r[octet.proof_artifact_library.fail_closed_equivalence]
- [x] [serial] Run targeted `octet-standards` proof/artifact tests, `cargo-octet` prove tests, and Cairn validation/gates. r[octet.proof_artifact_library.proof_binding]
