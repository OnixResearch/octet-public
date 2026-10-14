# Effect Closure Admission Specification Delta

## ADDED Requirements

### Requirement: Kamacite semantic-emission profiles are explicit

r[octet.effect_closure.kamacite_mapping]

Octet MUST define an opt-in Nickel profile for one effect-closure profile and one canonical Kamacite closed effect summary.

Each semantic-emission rule MUST bind one exact function address or call-site, provider identity, Kamacite operation identity, evidence reference, and non-claims.

Ambient effect dispositions MUST remain a separate policy section.

#### Scenario: Exact semantic emission is declared

- GIVEN a reviewed emission site has one exact Kamacite operation identity and the closed row contains that identity
- WHEN semantic-emission validation runs
- THEN the rule MUST pass with the `semantic-emission` class.

#### Scenario: Semantic emission is ambiguous

- GIVEN one emission site matches multiple rules or one rule uses a broad unresolved scope
- WHEN semantic-emission validation runs
- THEN validation MUST fail with a deterministic ambiguous-semantic-emission diagnostic.

### Requirement: Ambient effects cannot become semantic emissions

r[octet.effect_closure.kamacite_dispositions]

Existing strict-profile ambient effect results MUST remain authoritative for Octet admission.

The integration MUST NOT discharge, rename, or convert a denied ambient effect into a Kamacite semantic operation.

The V1 profile MAY classify only `alloc` as `profile-local`. All other ambient effect classes MUST remain denied under `deterministic-core`.

#### Scenario: Allocation remains profile-local

- GIVEN the selected strict profile permits allocation and the reviewed profile classifies it as profile-local
- WHEN Octet compares closure and semantic-emission evidence with the Kamacite row
- THEN allocation MUST remain visible without becoming a semantic operation or disappearing from evidence.

#### Scenario: Ambient clock cannot become a semantic emission

- GIVEN selected source reads an ambient clock and a mapping rule names a semantic time operation
- WHEN direct closure and semantic-emission admission run
- THEN direct closure MUST remain denied and the mapping MUST NOT discharge or rename the clock effect.

#### Scenario: Unknown effect fails

- GIVEN closure evidence contains `unknown` or an unsupported ambient disposition
- WHEN comparison runs
- THEN admission MUST fail with a deterministic unmapped-ambient-effect diagnostic.

### Requirement: Comparison preserves ambient, emission, and declaration authority

r[octet.effect_closure.kamacite_comparison]

Octet MUST compare verified closure and semantic-emission evidence with the supplied Kamacite projection without rewriting any parent artifact.

The comparison MUST classify rows as `emitted-and-declared`, `profile-local`, `declared-not-emitted`, `emitted-not-declared`, `stale-identity`, `ambiguous-emission`, or `unknown`.

Only `emitted-and-declared`, `profile-local`, and `declared-not-emitted` MAY appear in an admitted result.

#### Scenario: Emitted undeclared operation fails

- GIVEN an exact semantic-emission site names an operation absent from the supplied closed effect row
- WHEN comparison runs
- THEN admission MUST fail with an `emitted-not-declared` result.

#### Scenario: Declared operation is not emitted

- GIVEN the Kamacite row contains an operation that selected Octet emission evidence does not include
- WHEN comparison runs
- THEN the result MUST record `declared-not-emitted` without claiming dead code, unreachable behavior, or complete coverage.

### Requirement: Kamacite comparison is an optional admission extension

r[octet.effect_closure.kamacite_cli]

`cargo octet admit --profile deterministic-core --closure` MAY accept `--kamacite-effect-profile PATH` after direct checks and effect closure pass.

The option MUST consume a projection bound to canonical Kamacite Preserves identities. It MUST NOT invoke Kamacite or change default admission behavior.

#### Scenario: Optional profile emits a handoff

- GIVEN direct checks, closure, semantic-emission admission, identity checks, and comparison all pass
- WHEN the operator supplies the Kamacite effect profile
- THEN Octet MUST emit the comparison and handoff artifacts with successful status.

#### Scenario: Default admission remains unchanged

- GIVEN the operator does not supply a Kamacite effect profile
- WHEN Octet runs existing check, admission, and artifact verification commands
- THEN arguments, behavior, receipts, exit status, and claim boundaries MUST remain compatible.

### Requirement: Kamacite handoff is deterministic and non-authoritative

r[octet.effect_closure.kamacite_handoff]

A successful comparison MUST emit a BLAKE3-bound handoff projection with exact closure, emission, profile, effect-summary, and operation identities.

The handoff MUST also bind function-address or provider scope, comparison class, diagnostics, producer role, and non-claims.

#### Scenario: Verified handoff is stable

- GIVEN identical closure, emission, profile, effect summary, and producer inputs
- WHEN Octet emits the handoff twice
- THEN canonical rows and the handoff identity MUST match.

#### Scenario: Parent identity is tampered

- GIVEN a handoff changes a closure, emission, profile, summary, operation, or producer identity after emission
- WHEN artifact verification runs
- THEN verification MUST fail with a deterministic parent-identity diagnostic.

### Requirement: Kamacite comparison claims remain bounded

r[octet.effect_closure.kamacite_boundary]

A passing comparison MUST prove only bounded agreement between selected Octet evidence and supplied Kamacite declaration artifacts.

It MUST NOT prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility.

#### Scenario: Semantic determinism claim fails

- GIVEN a handoff, receipt, report, help text, or document claims that comparison proves semantic or runtime determinism
- WHEN boundary validation runs
- THEN validation MUST fail with a deterministic effect-comparison-overclaim diagnostic.

### Requirement: Kamacite comparison has positive and negative fixtures

r[octet.effect_closure.kamacite_fixtures]

Octet MUST include positive, negative, property, compatibility, and frozen Kamacite fixtures for the emission and comparison path.

#### Scenario: Frozen Kamacite cohort drifts

- GIVEN a Kamacite schema, profile, effect-summary, operation identity, or non-claim field changes
- WHEN Octet validates the frozen cohort
- THEN validation MUST fail until an explicit reviewed cohort update binds the new identities.
