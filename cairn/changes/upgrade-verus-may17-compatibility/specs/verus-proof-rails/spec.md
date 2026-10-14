# Verus proof rails delta

## ADDED Requirements

### Requirement: Production verifier upgrades require compatibility evidence [r[octet.verus_proof_rails.production_upgrade_compatibility]]

Octet MUST bind a production verifier upgrade to an exact published asset, matching Rust closure, package identities, positive and negative Octet proof rails, and explicit registered-consumer compatibility evidence. Package availability MUST NOT be treated as proof compatibility.

#### Scenario: Candidate release passes exact rails

- GIVEN a SHA-256-pinned Verus release and matching Rust toolchain
- WHEN Octet proof rails and a registered consumer's exact proof receipt pass
- THEN the profile MAY select that release and mark that consumer `native`
- AND the row MUST retain a non-transfer rationale

#### Scenario: Adapted corpus source passes

- GIVEN a registered consumer binds an immutable upstream source and a separate compatibility patch identity
- WHEN its native verifier command exits zero without increasing assumptions or trusted markers
- THEN Octet MAY consume that result only as an adapted-source compatibility result
- AND MUST NOT report it as proof of the unmodified upstream revision

#### Scenario: Registered consumer remains blocked

- GIVEN the candidate package builds but a registered consumer proof fails
- WHEN compatibility rows are exported
- THEN that consumer MUST remain `local-pin`, `unsupported`, or another truthful non-native status
- AND the package build MUST NOT authorize downstream promotion
