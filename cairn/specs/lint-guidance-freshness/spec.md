# Lint Guidance Freshness Specification

## Purpose

Defines the `lint-guidance-freshness` capability.

## Requirements

### Requirement: Guidance enforcement-tier freshness

r[octet.lint_guidance_freshness.enforcement_tiers] Octet narrative guidance that names lint enforcement tiers SHOULD be checked against the live lint catalog when it references shipped lint identities.

#### Scenario: Guidance row matches live lint identity

r[octet.lint_guidance_freshness.enforcement_tiers.row]

- GIVEN a guidance document names a lint identity present in the live lint inventory
- WHEN guidance freshness validation runs
- THEN the guidance SHOULD describe that lint as shipped, advisory, warn, deny, or future according to the live catalog and accepted rollout contract.

### Requirement: Shipped lints are not future-only guidance

r[octet.lint_guidance_freshness.shipped_lints] Guidance pages MUST NOT describe a shipped lint as only a future candidate unless the text explicitly distinguishes a separate future expansion.

#### Scenario: Borrowed argument guidance is current

r[octet.lint_guidance_freshness.shipped_lints.borrowed_argument_types]

- GIVEN `borrowed_argument_types` is registered in the live lint inventory with default level `warn`
- WHEN `docs/rust-idiom-guidance.md` is checked
- THEN the `Borrowed argument type` row MUST NOT describe it as only a future lint candidate.
- AND the row SHOULD point readers to the shipped lint or its fixture-backed enforcement tier.

#### Scenario: Stale future-only wording is rejected

r[octet.lint_guidance_freshness.shipped_lints.stale]

- GIVEN a guidance fixture describes a live lint as `future lint candidate`
- WHEN guidance freshness validation runs
- THEN validation MUST fail with a diagnostic naming the lint and the stale document section.
