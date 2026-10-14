## ADDED Requirements

### Requirement: Nominal Type Boundary Guidance [r[nominal-type-boundary-guidance]]

The system MUST document when Rust code should use nominal newtypes instead of interchangeable primitive or structural types for IDs, units, secrets, capability tokens, protocol states, and authority-bearing values.

#### Scenario: Identity values use nominal wrappers [r[nominal-type-boundary-guidance-scenario-identity-values-use-nominal-wrappers]]

- GIVEN a public API accepts an account ID, job ID, token, unit-bearing value, or secret
- WHEN type-boundary guidance is applied
- THEN the API prefers a named wrapper over a raw primitive when confusion would be safety-relevant

#### Scenario: Structural records remain intentional [r[nominal-type-boundary-guidance-scenario-structural-records-remain-intentional]]

- GIVEN two data shapes are intentionally interchangeable plain data
- WHEN structural guidance is applied
- THEN a plain record or tuple remains acceptable with documented interchangeability

### Requirement: Primitive Alias Lint Staging [r[primitive-alias-lint-staging]]

The system MUST classify future primitive-alias or missing-newtype lint candidates as staged until fixtures prove that warnings identify safety-relevant ambiguity.

#### Scenario: Primitive lint needs evidence [r[primitive-alias-lint-staging-scenario-primitive-lint-needs-evidence]]

- GIVEN a future checker flags raw IDs or unit values
- WHEN hard-gate promotion is considered
- THEN positive and negative fixtures prove it avoids harmless local primitives
