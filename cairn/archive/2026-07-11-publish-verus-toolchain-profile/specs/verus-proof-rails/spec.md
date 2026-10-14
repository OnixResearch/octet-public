# Verus Proof Rails Specification Delta

## ADDED Requirements

### Requirement: Typed Verus toolchain compatibility profile
r[octet.verus_proof_rails.toolchain_profile] Octet MUST publish its reviewed Verus verifier, solver, Rust toolchain, formatter, fixed-output fetch identities, supported targets, claim scope, and non-claims through a typed Nickel profile with deterministic exported runtime data.

#### Scenario: Current profile exports deterministically
- GIVEN the checked Nickel profile and generated runtime export
- WHEN profile freshness validation runs
- THEN the export MUST match the Nickel source byte-for-byte after deterministic generation
- AND malformed or incomplete profiles MUST fail before verifier or formatter execution.

### Requirement: Compatibility rows do not transfer proof claims
r[octet.verus_proof_rails.toolchain_profile.compatibility] Compatibility rows MUST distinguish exact native use, intentional local pins, unsupported targets, and semantic-reference-only snapshots. A row MUST NOT imply that proof receipts, verifier soundness, or semantic coverage transfer across versions.

#### Scenario: Different consumer pin remains explicit
- GIVEN a consumer uses a verifier release different from the Octet profile
- WHEN the compatibility profile is reviewed
- THEN the consumer MUST remain labeled `local-pin` or otherwise non-native
- AND its proof artifacts MUST retain their own verifier, solver, toolchain, source, and receipt identities.

### Requirement: Pinned Verus formatter rail
r[octet.verus_proof_rails.verusfmt] Octet SHOULD package a pinned `verusfmt` release and require Octet-owned Verus proof sources to pass `verusfmt --check --verus-only` without using the formatter self-update path.

#### Scenario: Formatted proof sources pass
- GIVEN Octet-owned Verus proof files and the pinned formatter package
- WHEN the formatter rail runs
- THEN every selected proof file MUST pass without mutation
- AND the formatter identity MUST name the exact release and fetched asset.

### Requirement: Formatter failures fail closed
r[octet.verus_proof_rails.verusfmt.fail_closed] The formatter rail MUST include positive and negative fixtures and MUST reject unformatted Verus input with a non-zero result.

#### Scenario: Unformatted fixture is rejected
- GIVEN a deliberately unformatted valid Verus fixture
- WHEN `verusfmt --check --verus-only` runs
- THEN the command MUST fail
- AND the failure MUST remain formatting evidence only, not a failed or passed proof claim.

### Requirement: Toolchain profile final validation
r[octet.verus_proof_rails.toolchain_profile.final_validation] The change MUST preserve production Verus positive and negative behavior, policy validation, formatter fixture behavior, and profile freshness before archive.

#### Scenario: Focused validation passes
- GIVEN the completed profile and formatter package
- WHEN focused Nix, policy, proof, formatter, and Cairn checks run
- THEN positive checks MUST pass and negative fixtures MUST fail as declared
- AND starter `cargo octet check` behavior MUST remain independent of Verus tooling.
