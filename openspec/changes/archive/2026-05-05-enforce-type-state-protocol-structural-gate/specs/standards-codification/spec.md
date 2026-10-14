## MODIFIED Requirements

### Requirement: Nominal Type Boundary Guidance [r[nominal-type-boundary-guidance]]

The system MUST document when Rust code should use nominal newtypes instead of interchangeable primitive or structural types for IDs, units, secrets, capability tokens, protocol states, and authority-bearing values.

For declared type-state protocol surfaces, the system MUST export structural checker obligations that reject configured raw state aliases and raw transition bypasses while accepting nominal marker-state APIs with visible transition methods.

#### Scenario: Identity values use nominal wrappers [r[nominal-type-boundary-guidance-scenario-identity-values-use-nominal-wrappers]]

- GIVEN a public API accepts an account ID, job ID, token, unit-bearing value, or secret
- WHEN type-boundary guidance is applied
- THEN the API prefers a named wrapper over a raw primitive when confusion would be safety-relevant

#### Scenario: Structural records remain intentional [r[nominal-type-boundary-guidance-scenario-structural-records-remain-intentional]]

- GIVEN two data shapes are intentionally interchangeable plain data
- WHEN structural guidance is applied
- THEN a plain record or tuple remains acceptable with documented interchangeability

#### Scenario: Compliant nominal marker-state API passes [r[nominal-type-boundary-guidance-scenario-compliant-typestate-passes]]

- GIVEN a generated policy artifact declares type-state source scopes, required state marker names, and required transition method names
- WHEN `tigerstyle-standards check --policy-artifacts-dir` scans a compliant source root with nominal marker states and transition methods
- THEN the type-state structural policy emits no diagnostics for that compliant source root

#### Scenario: Raw protocol bypass fails structurally [r[nominal-type-boundary-guidance-scenario-raw-protocol-bypass-fails]]

- GIVEN a generated policy artifact declares raw state alias and raw transition bypass patterns
- WHEN `tigerstyle-standards check --policy-artifacts-dir` scans a declared source root containing one of those raw bypasses
- THEN the type-state structural policy emits an error diagnostic naming the configured bypass pattern and exits nonzero when hard policies fail

### Requirement: Nickel Mechanism Contracts [r[nickel-mechanism-contracts]]

The system MUST define mechanism-specific Nickel contracts for crate boundaries, capability APIs, bounded primitives, type-state protocols, proof rails, and seed corpora.

Mechanism contracts MUST include the fields needed by Rust checkers to validate the corresponding structural or verified claim without inferring policy intent from prose.

For bounded primitive mechanisms, the exported contract MUST include approved bounded APIs or wrapper scopes, forbidden unbounded APIs, capacity-witness requirements, and backpressure or overflow behavior requirements.

For type-state mechanisms, Nickel contracts MUST support deterministic exported fields for source scopes, raw state alias patterns, raw transition patterns, required state marker names, and required transition method names.

#### Scenario: Bounded queue artifact exports checker obligations [r[nickel-mechanism-contracts.scenario.bounded-queue-artifact-obligations]]

- GIVEN `tigerstyle.embedded.bounded-queues` declares `mechanism.kind = 'bounded_primitive`
- WHEN generated policy artifacts are exported
- THEN the bounded queue policy artifact includes forbidden unbounded constructors, approved bounded constructors or wrapper scopes, required capacity-witness fields, and required backpressure or overflow behavior fields
- AND the artifact preserves `tigerstyle.embedded.bounded-queues` as the policy ID consumed by `tigerstyle-standards check --policy-artifacts-dir`

#### Scenario: Type-state obligations export deterministically [r[nickel-mechanism-contracts-scenario-type-state-obligations-export]]

- GIVEN the policy registry declares a `type-state` mechanism for nominal type boundaries
- WHEN generated policy artifacts are exported from the Nickel registry
- THEN the policy JSON and checker config include the configured type-state source scopes, raw bypass patterns, state marker names, and transition method names

### Requirement: Structural Mechanism Validation [r[structural-mechanism-validation]]

The system MUST validate claimed structural enforcement mechanisms before accepting a policy as structurally enforced.

Structural validation MUST cover at least no-std core crates, shell-only side effects, approved capability constructors, bounded primitive use, type-state protocol surfaces, and crate dependency direction.

For bounded queue policies, structural validation MUST reject configured unbounded queue constructors in selected source roots and MUST accept only approved bounded constructors or wrappers with explicit capacity and backpressure witnesses.

For effect capability policies, structural validation MUST reject configured ambient authority APIs in declared core source scopes while accepting shell-owned constructors and injected capability or effect-data APIs.

For type-state protocol policies, structural validation MUST reject configured raw state aliases and raw transition shortcuts in declared source scopes while accepting nominal marker-state wrappers with declared transition methods.

#### Scenario: Artifact-backed checker rejects configured unbounded constructors [r[structural-mechanism-validation.scenario.bounded-queue-rejects-unbounded-constructors]]

- GIVEN a generated policy artifact for `tigerstyle.embedded.bounded-queues` lists Tokio, Crossbeam, or std unbounded channel constructors as forbidden APIs
- WHEN `tigerstyle-standards check --policy-artifacts-dir <dir> --family embedded` scans a source root containing one configured unbounded constructor
- THEN the checker exits non-zero
- AND the diagnostic includes policy ID `tigerstyle.embedded.bounded-queues`, structural enforcement mode, bounded primitive mechanism, source path, line when available, and remediation to use an explicit bounded queue with backpressure

#### Scenario: Bounded construction with witness passes [r[structural-mechanism-validation.scenario.bounded-queue-witness-passes]]

- GIVEN source code constructs queues only through approved bounded APIs or approved wrapper scopes
- AND each construction has an explicit capacity constant, capacity argument, or configured wrapper witness plus backpressure or overflow behavior
- WHEN the artifact-backed embedded structural checker scans the source root
- THEN the checker reports zero bounded queue diagnostics
- AND policy evaluation metadata still records that `tigerstyle.embedded.bounded-queues` was evaluated as a structural bounded primitive policy

#### Scenario: Missing capacity witness remains a structural finding [r[structural-mechanism-validation.scenario.bounded-queue-missing-witness-fails]]

- GIVEN source code hides queue construction behind a wrapper or helper that does not expose a configured capacity witness or backpressure behavior
- WHEN the artifact-backed embedded structural checker scans the source root
- THEN the checker reports a structural finding for `tigerstyle.embedded.bounded-queues`
- AND the remediation names the missing capacity or backpressure witness rather than reporting a generic lint failure

#### Scenario: Downstream mutation fails the policy artifact consumer check [r[structural-mechanism-validation.scenario.bounded-queue-downstream-mutation-fails]]

- GIVEN Aspen pins a Tigerstyle flake exposing `lib.mkPolicyArtifactConsumerCheck` and checks selected embedded roots with `checks.x86_64-linux.tigerstyle-policy-artifacts`
- WHEN a scratch Aspen copy inserts exactly one configured unbounded queue constructor inside a checked root
- THEN the focused Aspen policy artifact check exits non-zero
- AND the saved transcript names `tigerstyle.embedded.bounded-queues`, the mutated file, and the single expected mutation diagnostic before the scratch mutation is removed or discarded

#### Scenario: Focused Tigerstyle checks pass [r[structural-mechanism-validation-scenario-focused-typestate-checks-pass]]

- GIVEN the type-state structural checker, fixtures, policy registry, and generated artifact export are changed together
- WHEN the focused Rust, Nickel/Nix, OpenSpec, docs, and whitespace checks run
- THEN every focused check exits successfully and the saved transcript names the commands used
