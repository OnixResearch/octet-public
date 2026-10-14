## ADDED Requirements

### Requirement: Raw boolean parameter policy is explicit

r[octet.structure.raw_bool_parameter.policy] Octet MUST define the covered visibility, callable, type, trait, expansion, exception, and diagnostic rules for raw boolean parameters.

#### Scenario: Policy loads

- GIVEN the policy names public API scope, private count scope, direct `bool`, exclusions, exceptions, and non-claims
- WHEN policy validation runs
- THEN it MUST produce one deterministic policy identity

#### Scenario: Policy weakens full-catalog enforcement

- GIVEN policy permits a warning budget, finding baseline, broad suppression, or silent unknown scope
- WHEN policy validation runs
- THEN validation MUST fail before lint execution

### Requirement: Public APIs avoid direct raw booleans

r[octet.structure.raw_bool_parameter.public_api] The lint MUST report each direct `bool` parameter on covered public free functions, inherent methods, and crate-owned trait declarations.

#### Scenario: Public function takes a boolean

- GIVEN a covered public function declares one direct `bool` parameter
- WHEN the structure lint runs
- THEN it MUST emit `raw_bool_parameter` at that parameter
- AND help MUST request a domain enum, option record, typestate value, or nominal wrapper

#### Scenario: Public API uses a domain type

- GIVEN the parameter uses a domain enum, option record, typestate value, or nominal wrapper
- WHEN the lint runs
- THEN it MUST emit no raw-boolean finding for that parameter

### Requirement: Multiple private booleans are reported

r[octet.structure.raw_bool_parameter.private_api] The lint MUST report a non-public callable when its direct boolean-parameter count reaches the named minimum.

#### Scenario: Private callable has multiple direct booleans

- GIVEN a covered private function, method, or closure reaches the named direct-boolean count
- WHEN the lint runs
- THEN it MUST emit a stable ambiguity finding

#### Scenario: Private helper has one predicate flag

- GIVEN a private helper has one direct boolean parameter and no other covered boolean parameter
- WHEN the lint runs
- THEN this lint MUST remain silent
- AND other naming or API rules MAY still apply

### Requirement: Compatibility signatures remain bounded

r[octet.structure.raw_bool_parameter.compatibility] The lint MUST skip generated expansion output and externally owned trait implementation signatures. It MUST keep crate-owned trait declarations in scope.

#### Scenario: External trait fixes the method signature

- GIVEN an implementation method matches a trait owned by another crate
- WHEN the lint runs
- THEN it MUST emit no finding for the required signature

#### Scenario: Local trait declares a boolean parameter

- GIVEN a public trait owned by the current crate declares one direct boolean parameter
- WHEN the lint runs
- THEN it MUST emit `raw_bool_parameter`

### Requirement: Diagnostics require domain judgment

r[octet.structure.raw_bool_parameter.diagnostics] Diagnostics MUST identify the callable and parameter and MUST NOT offer a machine-applicable type rewrite.

#### Scenario: Finding is rendered

- GIVEN one covered raw boolean parameter
- WHEN the diagnostic is emitted
- THEN it MUST explain call-site ambiguity and list bounded domain-model alternatives
- AND it MUST state that boolean returns, fields, and predicate values are outside this finding

### Requirement: Exceptions stay narrow

r[octet.structure.raw_bool_parameter.exceptions] An unavoidable compatibility API MAY use a narrow source expectation with a specific reason and named owner.

#### Scenario: Narrow compatibility expectation is valid

- GIVEN one exact API cannot change because of a recorded external compatibility contract
- WHEN its local expectation includes a specific reason and owner
- THEN policy MAY admit that exact scope

#### Scenario: Broad suppression is attempted

- GIVEN a crate, module, workspace, baseline, or warning budget suppresses the lint
- WHEN policy validation runs
- THEN Octet MUST reject the suppression

### Requirement: Catalog registration is complete

r[octet.structure.raw_bool_parameter.catalog] The lint MUST have registration, group, metadata, source, profile, docs, fixture, and deny-all coverage before activation.

#### Scenario: Full catalog is checked

- GIVEN the lint is enabled for release
- WHEN catalog freshness and deny-all checks run
- THEN every required registration and artifact MUST exist and agree

### Requirement: Clippy evidence remains independent

r[octet.structure.raw_bool_parameter.clippy_evidence] Octet MUST preserve `clippy::fn_params_excessive_bools` as an independent co-reporting finding.

#### Scenario: Both tools report one API

- GIVEN one callable satisfies both lint rules
- WHEN Octet collects native and Clippy evidence
- THEN each finding MUST retain its own checker, rule, message, and identity

### Requirement: Verification covers accepted and rejected shapes

r[octet.structure.raw_bool_parameter.verification] The implementation MUST include positive and negative UI fixtures for every covered visibility, callable, trait, expansion, and exception boundary.

#### Scenario: Fixture matrix runs

- GIVEN the complete positive and negative fixture matrix
- WHEN UI validation runs
- THEN each positive fixture MUST remain clean
- AND each negative fixture MUST match its stable diagnostic snapshot

### Requirement: Migration precedes strict activation

r[octet.structure.raw_bool_parameter.migration] Octet MUST migrate the declared self and consumer scope before it enables the lint in the full deny-all catalog.

#### Scenario: Declared migration scope is clean

- GIVEN Octet and selected consumers have no unjustified findings
- WHEN full-catalog activation occurs
- THEN the deny-all gate MUST reject every future finding as an error

#### Scenario: Migration retains a warning budget

- GIVEN any declared consumer relies on a warning budget or finding baseline for this lint
- WHEN activation review runs
- THEN activation MUST fail

### Requirement: External style reference remains bounded

r[octet.structure.raw_bool_parameter.reference_boundary] Octet MAY adapt explicit boolean-parameter and state-modeling guidance from the MIT-licensed Radiance style guide.

#### Scenario: Guidance is published

- GIVEN Octet documents the new rule
- WHEN reference review runs
- THEN it MUST name the exact Radiance revision and selected guidance
- AND it MUST reject unrelated helper-size and panic guidance from the Octet rule
