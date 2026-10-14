# Namespace Compatibility Specification

## Purpose

Defines the `namespace-compatibility` capability.

## Requirements

### Requirement: Canonical namespace consistency

r[octet.namespace_compatibility.canonical] Octet MUST document and implement `tigerstyle::` as the canonical namespace for source attributes, domain group attributes, and emitted lint group metadata for the current release line while preserving Octet product, command, and config naming.

#### Scenario: Docs match live namespace

r[octet.namespace_compatibility.canonical.docs]

- GIVEN README, lint reference, templates, and consumer guide describe lint source attrs or domain groups
- WHEN namespace freshness validation runs
- THEN those docs MUST match the `tigerstyle::` namespace accepted by live lint declarations and UI fixtures.

#### Scenario: Runner output uses canonical group names

r[octet.namespace_compatibility.canonical.runner]

- GIVEN `cargo-octet` emits JSONL or SARIF findings
- WHEN a finding includes a group name
- THEN the group name MUST use the documented `tigerstyle::` namespace unless the output explicitly labels a compatibility alias.

### Requirement: Compatibility aliases

r[octet.namespace_compatibility.aliases] Octet SHOULD retain tested bare lint-name compatibility aliases and legacy config sections until a documented removal boundary is reached.

#### Scenario: Bare lint attr remains supported

r[octet.namespace_compatibility.aliases.legacy_attr]

- GIVEN a consumer uses a documented supported bare lint-name source attribute
- WHEN Octet lints the consumer crate
- THEN the attribute SHOULD behave like the canonical namespace or emit a documented deprecation diagnostic.

#### Scenario: Unsupported namespace fails clearly

r[octet.namespace_compatibility.aliases.unsupported]

- GIVEN a consumer uses an unsupported namespace spelling
- WHEN Octet lints the consumer crate
- THEN validation SHOULD fail or warn with a diagnostic that identifies the unsupported namespace and the canonical replacement.

### Requirement: Escape-hatch behavior preservation

r[octet.namespace_compatibility.escape_hatches] Namespace migration MUST preserve custom allow/escape-hatch semantics for lints that require local justification or special-case handling.

#### Scenario: Justified todo escape still works

r[octet.namespace_compatibility.escape_hatches.justified]

- GIVEN a temporary `todo!()` stub uses a supported allow attribute with an adjacent justification comment
- WHEN `no_todo` runs after namespace migration
- THEN the justified stub MUST keep the documented behavior.

#### Scenario: Unjustified allow still fails

r[octet.namespace_compatibility.escape_hatches.unjustified]

- GIVEN an allow attribute for a lint with custom justification semantics lacks the required local reason
- WHEN the lint runs after namespace migration
- THEN Octet MUST still emit the custom unjustified-allow diagnostic rather than silently suppressing the lint.

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
