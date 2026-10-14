## Phase 1: Implementation

- [x] [serial] r[octet.verus_proof_rails.proof_package_lock] Inventory proof-rail identity fields already present in manifests, receipts, proof cards, and artifact provenance.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock] Define a proof package lock DTO and generated contract or policy registry entry for selected production proof rails.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.offline] Add pure lock validation over local manifest, source, proof, verifier, solver, toolchain, closure, command, environment, pass-signal, and trust-boundary identities.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.receipt_match] Require promotion-grade receipts to match the lock where a lock is declared.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.starter_independent] Keep starter check paths independent from proof package locks.

## Phase 2: Verification

- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.valid] Verify a valid lock validates deterministically and names the exact proof package material.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.stale] Verify stale source hashes, stale proof hashes, wrong verifier or solver identity, mismatched command argv, invalid environment allowlist, malformed closure graph, and missing trust-boundary text fail closed.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.receipt_match] Verify a receipt that does not match the declared lock cannot emit `formal_proof` provenance.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.offline] Verify lock validation runs offline from checked-in artifacts and does not fetch network resources.
- [x] [serial] r[octet.verus_proof_rails.proof_package_lock.starter_independent] Verify ordinary `cargo octet check` remains unaffected when no proof rail is selected.
- [x] [serial] Run focused proof/provenance/artifact tests plus Cairn validate and proposal/design/tasks gates. [depends:cairn-gates]
