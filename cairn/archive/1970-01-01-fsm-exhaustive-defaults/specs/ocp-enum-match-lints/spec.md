# Ocp Enum Match Lints Delta

## ADDED Requirements

### Requirement: FSM exhaustive-default profile
r[octet.ocp_enum_match.fsm_exhaustive_defaults.profile] Octet MUST provide an FSM exhaustiveness profile that identifies reviewed FSM state/event enums, pure transition cores, generated-adapter exclusions, and receipt export settings through deterministic config and/or marker attributes.

#### Scenario: Marked transition core enters FSM profile
r[octet.ocp_enum_match.fsm_exhaustive_defaults.profile.marked]
- GIVEN a Rust transition core is in a configured FSM source scope or carries the configured FSM transition marker
- WHEN Octet checks enum matches in that core
- THEN Octet MUST apply the FSM exhaustive-default policy instead of the ordinary same-crate enum evolution advisory.

#### Scenario: Unmarked enum keeps ordinary OCP behavior
r[octet.ocp_enum_match.fsm_exhaustive_defaults.profile.unmarked]
- GIVEN a same-crate enum match is outside FSM scopes and has no FSM marker
- WHEN Octet checks the match
- THEN existing `fragile_exhaustive_enum_match` and `catch_all_on_enum` behavior MUST remain unchanged.

### Requirement: FSM transition cores deny catch-all enum arms
r[octet.ocp_enum_match.fsm_exhaustive_defaults.wildcards] Octet MUST diagnose `_` or equivalent broad catch-all enum match arms inside FSM transition cores because they can silently accept future state or event variants.

#### Scenario: Wildcard transition arm is rejected
r[octet.ocp_enum_match.fsm_exhaustive_defaults.wildcards.reject]
- GIVEN a marked FSM transition core matches a state enum, event enum, or state/event tuple with a `_` arm
- WHEN the FSM profile runs
- THEN Octet MUST emit a diagnostic naming the hidden future-variant risk and require explicit state/event handling or an out-of-core generated-adapter exception.

#### Scenario: Explicit denial arm is accepted
r[octet.ocp_enum_match.fsm_exhaustive_defaults.wildcards.explicit_denial]
- GIVEN a marked FSM transition core names each state/event case that denies and returns a typed denial reason without a wildcard
- WHEN the FSM profile runs
- THEN Octet MUST NOT flag the denial merely because it rejects a transition.

### Requirement: Marked FSM enums are deliberately closed
r[octet.ocp_enum_match.fsm_exhaustive_defaults.sealed] Octet MUST treat marked FSM state/event enums as deliberately closed for transition-core matching so exhaustive matches over those enums do not trigger fragile local enum diagnostics.

#### Scenario: Exhaustive FSM enum match is accepted
r[octet.ocp_enum_match.fsm_exhaustive_defaults.sealed.exhaustive]
- GIVEN a state or event enum is marked as an FSM enum or otherwise bound to a marked FSM transition core
- WHEN a transition core matches all variants without a wildcard
- THEN Octet MUST accept the exhaustive match as the desired review pressure for future variant additions.

#### Scenario: Missing closed-enum marker fails closed
r[octet.ocp_enum_match.fsm_exhaustive_defaults.sealed.missing_marker]
- GIVEN a transition core is marked as FSM but its state/event enum identity cannot be resolved as deliberately closed
- WHEN Octet evaluates the FSM profile
- THEN Octet MUST report a deterministic diagnostic requiring explicit FSM enum identity before accepting exhaustive-by-default evidence.

### Requirement: FSM coverage receipt
r[octet.ocp_enum_match.fsm_exhaustive_defaults.receipt] Octet MUST produce or normalize a bounded FSM exhaustiveness receipt that binds transition-core identity, state/event variant inventory identity, coverage summary identity, diagnostics, parent artifact identity, status, and required non-claims.

#### Scenario: Passing receipt binds coverage identities
r[octet.ocp_enum_match.fsm_exhaustive_defaults.receipt.pass]
- GIVEN a marked FSM transition core has no wildcard enum arms and its declared state/event inventory and coverage summary are current
- WHEN Octet emits FSM evidence
- THEN the receipt MUST bind the transition core, state/event inventories, coverage summary, diagnostics, artifact identity, and a passing status.

#### Scenario: Stale coverage fails closed
r[octet.ocp_enum_match.fsm_exhaustive_defaults.receipt.stale]
- GIVEN a receipt declares a state/event inventory or coverage summary identity that does not match the checked source facts
- WHEN Octet verifies or exports the receipt
- THEN the receipt MUST fail closed with a deterministic stale-coverage diagnostic.

### Requirement: FSM evidence boundaries remain visible
r[octet.ocp_enum_match.fsm_exhaustive_defaults.docs] Octet documentation and exported receipts MUST state that FSM exhaustiveness evidence is bounded source-shape evidence only.

#### Scenario: Consumer reads bounded claim
r[octet.ocp_enum_match.fsm_exhaustive_defaults.docs.boundary]
- GIVEN a consumer reads the FSM profile docs or Valence-compatible receipt output
- WHEN the supported claim is described
- THEN it MUST say the evidence does not prove behavioral correctness, protocol completeness, liveness, verifier soundness, release readiness, or exhaustive Rust coverage.

### Requirement: Positive and negative FSM fixtures
r[octet.ocp_enum_match.fsm_exhaustive_defaults.fixtures] Octet MUST include positive and negative fixtures for the FSM exhaustive-default profile before promotion.

#### Scenario: Positive fixtures cover valid FSMs
r[octet.ocp_enum_match.fsm_exhaustive_defaults.fixtures.positive]
- GIVEN fixtures contain marked FSM enums, pure transition cores, explicit denial cases, and current receipt identities
- WHEN the focused fixture suite runs
- THEN valid FSM fixtures MUST pass without wildcard or fragile-exhaustive diagnostics.

#### Scenario: Negative fixtures cover failure paths
r[octet.ocp_enum_match.fsm_exhaustive_defaults.fixtures.negative]
- GIVEN fixtures contain wildcard transition arms, missing FSM enum identity, stale coverage summaries, foreign enum catch-alls, or overclaiming receipt text
- WHEN the focused fixture suite runs
- THEN each fixture MUST fail with deterministic diagnostics naming the violated FSM profile rule.

### Requirement: FSM profile validation evidence
r[octet.ocp_enum_match.fsm_exhaustive_defaults.final_validation] The FSM exhaustive-default change MUST include focused UI/fixture validation, receipt validation, Octet policy validation, and Cairn validation before archive.

#### Scenario: Final validation is reviewable
r[octet.ocp_enum_match.fsm_exhaustive_defaults.final_validation.reviewable]
- GIVEN implementation is complete
- WHEN the change is prepared for archive
- THEN saved evidence MUST identify the focused commands, passing and failing fixture coverage, and Cairn proposal/design/tasks validation results.
