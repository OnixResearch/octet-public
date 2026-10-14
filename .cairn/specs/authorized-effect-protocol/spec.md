# Authorized Effect Protocol Specification

## Purpose

Defines the `authorized-effect-protocol` capability.

## Requirements

### Requirement: Outbound effects have explicit authority classification

r[octet.authorized_effect_protocol.classification]

Each effectful outbound adapter capability in an active project architecture policy MUST be `unprotected` with rationale or `protected` with a complete authorization protocol.

#### Scenario: Protected effect is fully classified

- GIVEN policy names the effect family, authority inputs, decision, witness, executor, observation, receipt, use cardinality, and non-claims
- WHEN policy validation runs
- THEN the protected classification MUST be accepted for compiler evaluation

#### Scenario: Effectful adapter is unclassified

- GIVEN compiler facts identify an outbound effect family with no matching classification
- WHEN `gate` or `admission` evaluation runs
- THEN the architecture phase MUST block with an unclassified-authority diagnostic

### Requirement: Protected protocol schema is typed

r[octet.authorized_effect_protocol.schema]

The Nickel project policy contract MUST define exact qualified names and scope bindings for every protected-effect protocol field.

#### Scenario: Complete protocol exports

- GIVEN all protected protocol names resolve to non-empty qualified names and declared project scopes
- WHEN policy export runs
- THEN generated runtime policy MUST preserve canonical field order and BLAKE3 identity

#### Scenario: Protocol omits an observation boundary

- GIVEN a protected protocol omits its observed-outcome type or receipt constructor
- WHEN policy validation runs
- THEN validation MUST fail with a deterministic missing-field diagnostic

### Requirement: Authorization witness is opaque

r[octet.authorized_effect_protocol.witness]

A declared authorization witness MUST prevent ambient construction through compiler-visible fields, derives, conversions, constructors, and duplication rules.

#### Scenario: Move-only opaque witness passes

- GIVEN witness fields are private, constructors are restricted, and a `consume` witness has no `Copy`, `Clone`, `Default`, `Deserialize`, broad conversion, or unsafe constructor path
- WHEN witness evaluation runs
- THEN the witness shape MUST pass

#### Scenario: Witness can be forged or replayed

- GIVEN the witness has public fields, a public unchecked constructor, forbidden derive, broad `From` or `Into`, unsafe construction, or forbidden duplication
- WHEN witness evaluation runs
- THEN strict modes MUST block with the exact forgery or replay path

#### Scenario: Reusable witness is declared

- GIVEN policy selects `borrow` cardinality with an explicit reusable-authority rationale and scope
- WHEN witness evaluation runs
- THEN borrowing MAY be accepted only within the declared scope

### Requirement: Witness originates from an approved decision

r[octet.authorized_effect_protocol.origin]

A witness supplied to a protected executor MUST have selected compiler provenance from an approved domain decision output.

#### Scenario: Approved decision produces witness

- GIVEN local compiler provenance links an executor witness argument to an approved decision output under matching policy and source identities
- WHEN origin evaluation runs
- THEN the witness origin MUST pass

#### Scenario: Witness origin is missing or ambiguous

- GIVEN provenance points to an unapproved constructor, merges incompatible origins, or cannot resolve the witness definition path
- WHEN strict evaluation runs
- THEN the protocol MUST block with an unauthorized or unknown-origin diagnostic

### Requirement: Protected executors require authorization

r[octet.authorized_effect_protocol.executor]

Every declared protected executor method and selected wrapper path MUST preserve the required witness parameter through the external effect site.

#### Scenario: Executor consumes valid witness

- GIVEN the application-owned executor port requires the witness and its adapter implementation preserves that signature
- WHEN an approved call reaches the external effect site
- THEN the executor obligation MUST pass

#### Scenario: Unguarded path reaches protected effect

- GIVEN an overload, helper, adapter method, or wrapper reaches the protected effect without the witness
- WHEN strict evaluation runs
- THEN the protocol MUST block with the unguarded call path

### Requirement: Planned and observed effect types remain distinct

r[octet.authorized_effect_protocol.receipts]

The protocol MUST keep planned effect, execution attempt, observed outcome, and execution receipt as distinct compiler-resolved types.

#### Scenario: Observation creates receipt

- GIVEN an adapter returns a declared observed outcome and the shell constructs a receipt that binds that outcome and attempt identity
- WHEN receipt-boundary evaluation runs
- THEN the receipt construction MAY pass

#### Scenario: Plan becomes receipt without observation

- GIVEN a constructor or conversion creates an execution receipt from a plan, witness, or attempt without a declared observed outcome
- WHEN strict evaluation runs
- THEN the protocol MUST block with a planned-effect promotion diagnostic

#### Scenario: Failure becomes success receipt

- GIVEN the observed outcome represents failed or denied execution
- WHEN a success-receipt constructor receives that outcome
- THEN strict evaluation MUST block

### Requirement: Denial performs no protected effect

r[octet.authorized_effect_protocol.denial]

A denied authority decision MUST produce no authorization witness and no selected protected executor call.

#### Scenario: Denied decision stays effect-free

- GIVEN the domain decision denies the requested protected action
- WHEN shell behavior is exercised with controlled ports
- THEN no witness MUST be available
- AND the protected executor fake MUST record zero calls

#### Scenario: Denial path calls executor

- GIVEN selected compiler or shell-test evidence shows a denied decision can call the protected executor
- WHEN protocol validation runs
- THEN the protocol MUST block

### Requirement: Protocol evidence preserves authority boundaries

r[octet.authorized_effect_protocol.evidence]

The architecture receipt MUST record classifications, witness facts, approved constructors, executor signatures, provenance, observations, findings, unknowns, and non-claims.

#### Scenario: Protocol result is repeatable

- GIVEN unchanged policy, source, compiler facts, role graph, and protocol inputs
- WHEN protocol evaluation runs twice
- THEN canonical protocol records and their BLAKE3 identity MUST match

#### Scenario: Passing protocol is overclaimed

- GIVEN output describes a passing protocol as proof of credential validity, revocation freshness, actor trust, business-policy correctness, external success, or receipt authenticity
- WHEN claim-boundary validation runs
- THEN validation MUST fail with an overclaim diagnostic

### Requirement: Authorized effect verification is adversarial

r[octet.authorized_effect_protocol.verification]

The implementation MUST include positive and negative policy, compiler, shell, receipt, provenance, replay, denial, and claim fixtures.

#### Scenario: Protocol fixture matrix runs

- GIVEN fixtures cover approval, denial, success, failure, forgery, forbidden derives, broad conversions, replay, unguarded wrappers, unknown origin, and receipt promotion
- WHEN focused validation runs
- THEN positive fixtures MUST pass deterministically
- AND negative fixtures MUST block with their declared diagnostic class
