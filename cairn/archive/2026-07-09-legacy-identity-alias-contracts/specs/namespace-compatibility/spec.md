# Namespace Compatibility Specification

## Purpose

Adds contracts for approved legacy identity aliases and retirement plans.

## Requirements

### Requirement: Legacy alias contract
r[octet.namespace_compatibility.legacy_alias_contracts.contract] Octet MUST support alias rows that map canonical project identities to legacy names with allowed surfaces, owner, status, retirement condition, and non-claims.

#### Scenario: Approved alias passes
r[octet.namespace_compatibility.legacy_alias_contracts.fixtures.positive]
- GIVEN a legacy alias appears only in an approved compatibility or archive-only surface
- WHEN alias validation runs
- THEN validation MUST pass and report the alias status.

#### Scenario: Unapproved alias fails
r[octet.namespace_compatibility.legacy_alias_contracts.fixtures.negative]
- GIVEN a legacy alias appears in an active unapproved surface or lacks retirement rationale
- WHEN alias validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Alias validation
r[octet.namespace_compatibility.legacy_alias_contracts.validation] Octet MUST classify alias occurrences as approved, deprecated, blocked, or archive-only.

#### Scenario: Report preserves non-claims
r[octet.namespace_compatibility.legacy_alias_contracts.docs]
- GIVEN an alias is approved for compatibility
- WHEN the report renders it
- THEN it MUST state that approval preserves compatibility only and does not make the legacy identity canonical.

### Requirement: Final validation
r[octet.namespace_compatibility.legacy_alias_contracts.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers alias classes
r[octet.namespace_compatibility.legacy_alias_contracts.final_validation.fixtures]
- GIVEN approved, deprecated, blocked, archive-only, and missing-rationale fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
