# Verus Proof Rails Specification Delta

## ADDED Requirements

### Requirement: Proof package lock records

r[octet.verus_proof_rails.proof_package_lock] Octet SHOULD support Lake-inspired proof package lock records for selected proof rails, binding intended proof-run material before promotion-grade formal-proof evidence is accepted.

#### Scenario: Valid proof package lock is deterministic

r[octet.verus_proof_rails.proof_package_lock.valid]

- GIVEN a selected production proof rail declares a proof package lock
- WHEN lock validation runs
- THEN the lock SHOULD bind the rail manifest identity, source artifact identities, proof artifact identities, verifier identity, solver identity, Rust toolchain identity, Nix closure or closure-graph identity, command argv, working-directory policy, environment allowlist, pass signal, trust-boundary text, schema version, and lock artifact identity.
- AND repeated validation of the same local inputs SHOULD produce byte-stable machine-readable output.

#### Scenario: Stale or incomplete lock fails closed

r[octet.verus_proof_rails.proof_package_lock.stale]

- GIVEN a proof package lock has a stale source hash, stale proof hash, wrong verifier identity, wrong solver identity, wrong toolchain identity, malformed closure graph, mismatched command argv, invalid environment allowlist, missing pass signal, missing trust-boundary text, unsupported schema version, or unsupported backend
- WHEN lock validation runs
- THEN validation MUST fail closed with deterministic diagnostics naming the rejected lock field.
- AND Octet MUST NOT use the stale lock to support `formal_proof` provenance.

#### Scenario: Receipt must match declared lock

r[octet.verus_proof_rails.proof_package_lock.receipt_match]

- GIVEN a proof rail declares a proof package lock and a verifier run emits a structured proof receipt
- WHEN proof receipt validation considers `formal_proof` provenance
- THEN the receipt MUST match the lock's source, proof, verifier, solver, toolchain, closure, command, environment, pass-signal, and trust-boundary identities.
- AND a receipt from a different or stale proof package MUST fail without making other proof rails ambiguous.

#### Scenario: Lock validation is offline

r[octet.verus_proof_rails.proof_package_lock.offline]

- GIVEN proof package lock validation runs in CI or release review
- WHEN local checked-in artifacts and declared paths are available
- THEN validation MUST use local files, generated manifests, and recorded identities only.
- AND it MUST NOT fetch network schemas, package registries, remote repositories, tool documentation, or verifier metadata.

#### Scenario: Starter check path remains independent

r[octet.verus_proof_rails.proof_package_lock.starter_independent]

- GIVEN a consumer runs ordinary `cargo octet check` with no selected proof rail
- WHEN proof package locks are absent or stale
- THEN the starter lint result MUST remain independent from proof package lock validation.
- AND missing proof package locks MUST NOT be reported as starter-path failures.
