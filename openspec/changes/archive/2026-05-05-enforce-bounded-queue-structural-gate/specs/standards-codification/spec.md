## MODIFIED Requirements

### Requirement: Nickel Mechanism Contracts [r[nickel-mechanism-contracts]]

The system MUST define mechanism-specific Nickel contracts for crate boundaries, capability APIs, bounded primitives, type-state protocols, proof rails, and seed corpora.

Mechanism contracts MUST include the fields needed by Rust checkers to validate the corresponding structural or verified claim without inferring policy intent from prose.

For bounded primitive mechanisms, the exported contract MUST include approved bounded APIs or wrapper scopes, forbidden unbounded APIs, capacity-witness requirements, and backpressure or overflow behavior requirements.

#### Scenario: Bounded queue artifact exports checker obligations [r[nickel-mechanism-contracts.scenario.bounded-queue-artifact-obligations]]

- GIVEN `tigerstyle.embedded.bounded-queues` declares `mechanism.kind = 'bounded_primitive`
- WHEN generated policy artifacts are exported
- THEN the bounded queue policy artifact includes forbidden unbounded constructors, approved bounded constructors or wrapper scopes, required capacity-witness fields, and required backpressure or overflow behavior fields
- AND the artifact preserves `tigerstyle.embedded.bounded-queues` as the policy ID consumed by `tigerstyle-standards check --policy-artifacts-dir`

### Requirement: Structural Mechanism Validation [r[structural-mechanism-validation]]

The system MUST validate claimed structural enforcement mechanisms before accepting a policy as structurally enforced.

Structural validation MUST cover at least no-std core crates, shell-only side effects, approved capability constructors, bounded primitive use, type-state protocol surfaces, and crate dependency direction.

For bounded queue policies, structural validation MUST reject configured unbounded queue constructors in selected source roots and MUST accept only approved bounded constructors or wrappers with explicit capacity and backpressure witnesses.

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
