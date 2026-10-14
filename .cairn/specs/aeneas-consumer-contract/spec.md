# Aeneas Consumer Contract Specification

## Purpose

Defines the `aeneas-consumer-contract` capability.

## Requirements

### Requirement: Typed consumer profile

r[octet.aeneas_consumer_contract.profile] Octet MUST define a closed Nickel consumer profile that binds the exact provider profile, compatibility matrix, consumer identity, source root, source paths, selected items, theorem paths, theorem IDs, requirement IDs, claim classes, approved statement BLAKE3 values, external models, trust inputs, resource policy, and non-claims.

#### Scenario: Complete consumer profile is accepted

- GIVEN a consumer profile supplies every required field and exact provider identity
- WHEN Nickel export and pure profile validation run
- THEN the generated consumer profile MUST be deterministic.
- AND it MUST preserve all consumer, source, theorem, trust, resource, and non-claim fields.

#### Scenario: Consumer profile weakens provider authority

- GIVEN a consumer changes provider pins, extraction policy, backend, network policy, or required non-claims
- WHEN profile validation runs
- THEN validation MUST fail before any provider process executes.

### Requirement: Exact source and selected-item binding

r[octet.aeneas_consumer_contract.source_selection] Octet MUST bind consumer source bytes, workspace-relative paths, selected item names, source spans, Charon declarations, LLBC, `translation.json` rows, and generated Lean declarations without using Charon `def_id` as durable identity.

#### Scenario: Exact selected item reaches the generated model

- GIVEN the consumer source and selected item match the measured Charon and Aeneas outputs
- WHEN source-selection validation runs
- THEN the selected item can proceed to theorem admission.
- AND durable identity MUST remain BLAKE3-bound to source, profile, span, and generated artifacts.

#### Scenario: Source or item selection is ambiguous

- GIVEN source bytes changed, a copied source replaced the declared source, item selection is broad, or multiple declarations match
- WHEN source-selection validation runs
- THEN validation MUST fail before theorem admission.

### Requirement: Existing command owns consumer execution

r[octet.aeneas_consumer_contract.execution] Octet MUST execute consumer profiles through existing `cargo octet prove` policy-artifact, rail-selection, source-root, provider-closure, bounded-process, environment-allowlist, and network-denial surfaces.

#### Scenario: Consumer rail uses the reviewed provider

- GIVEN a selected consumer rail references the reviewed provider and exact source root
- WHEN proof execution starts
- THEN the shell MUST use only the packaged provider executables and reviewed execution policy.

#### Scenario: Ambient provider fallback is requested

- GIVEN a consumer omits a packaged executable or attempts to use host rustc, Charon, Aeneas, Lean, or network access
- WHEN execution planning runs
- THEN planning MUST fail before the first stage starts.

### Requirement: Outcome classes are deterministic

r[octet.aeneas_consumer_contract.outcomes] Octet MUST classify consumer execution as `supported`, `blocked`, or `failed` from typed stage observations.

#### Scenario: Supported consumer proof completes

- GIVEN every selected stage succeeds within policy and all artifacts validate
- WHEN outcome classification runs
- THEN the outcome MUST be `supported`.

#### Scenario: Unsupported source is blocked

- GIVEN a reviewed Aeneas or Charon language boundary rejects the exact selected source
- WHEN outcome classification runs
- THEN the outcome MUST be `blocked` with the stage, construct, bounded diagnostic identity, and next discriminating action.

#### Scenario: Tool or policy error fails

- GIVEN package, process, environment, policy, artifact, or resource validation fails
- WHEN outcome classification runs
- THEN the outcome MUST be `failed`.
- AND Octet MUST NOT report a language-support conclusion.

### Requirement: Consumer theorem ownership is explicit

r[octet.aeneas_consumer_contract.theorem] Octet MUST keep generated Lean definitions separate from maintained consumer theorem source and bind each theorem to its requirement, claim class, approved statement identity, model inventory, and complete trust closure.

#### Scenario: Maintained theorem is admitted

- GIVEN the theorem mapping is complete, non-vacuous, and free of undeclared trust inputs
- WHEN theorem admission runs
- THEN the theorem can proceed to Lean validation.

#### Scenario: Theorem or trust mapping drifts

- GIVEN the theorem, requirement, claim, statement, model, assumption, axiom, opaque definition, or trust identity is missing or stale
- WHEN theorem admission runs
- THEN Octet MUST reject property promotion even if Lean accepts the source.

### Requirement: Consumer artifacts remain reviewable

r[octet.aeneas_consumer_contract.artifacts] Octet MUST emit consumer-bound receipts and proof cards that expose provider, profile, source, selected-item, theorem, requirement, trust, outcome, artifact, and non-claim identities.

#### Scenario: Consumer artifact bundle is valid

- GIVEN a supported consumer run emits complete untampered artifacts
- WHEN `cargo octet artifact verify` runs
- THEN artifact validation MUST accept the exact consumer bundle and scoped theorem claim.

#### Scenario: Artifact or role is stale

- GIVEN an artifact, identity, outcome, role, receipt link, or non-claim changes after execution
- WHEN artifact validation runs
- THEN validation MUST fail and MUST NOT retain the prior property disposition.

### Requirement: Valence handoff preserves consumer boundaries

r[octet.aeneas_consumer_contract.valence_handoff] Octet MUST export a verified consumer receipt through the existing Valence Aeneas chain adapter while preserving translation stages as recorded-only or boundary evidence.

#### Scenario: Verified consumer receipt exports

- GIVEN consumer artifact validation succeeds
- WHEN Valence export runs
- THEN the export MUST bind the consumer profile, source, stages, theorem, requirement, receipt, roles, and non-claims.

#### Scenario: Unverified consumer receipt cannot export property evidence

- GIVEN consumer artifact validation is absent or failed
- WHEN Valence export runs
- THEN Octet MUST refuse property-role export.

### Requirement: Existing proof rails remain compatible

r[octet.aeneas_consumer_contract.compatibility] Octet MUST preserve the shipped Aeneas fixture rail, existing Verus rails, and ordinary starter checks while adding the consumer contract.

#### Scenario: Existing proof rails retain behavior

- GIVEN the current Aeneas fixture, Verus fixtures, and starter commands
- WHEN their focused compatibility tests run
- THEN accepted outputs and expected failures MUST retain their declared outcomes.

### Requirement: Consumer failures have negative evidence

r[octet.aeneas_consumer_contract.negative_evidence] Octet MUST include positive, false-theorem, vacuous-theorem, profile-drift, source-drift, ambiguous-selection, shadow-source, extraction, translation, trust, process, budget, artifact, role, and export fixtures.

#### Scenario: Negative fixture matrix runs

- GIVEN the consumer contract implementation is ready
- WHEN the negative matrix runs
- THEN every fixture MUST fail at its declared gate with its expected diagnostic class.
- AND negative fixtures MUST NOT emit property evidence.

### Requirement: Roadmap and claims remain bounded

r[octet.aeneas_consumer_contract.boundaries] Octet documentation MUST keep Aeneas consumer execution in Octet, theorem campaigns in consumers, evidence identity in Valence, build evidence in Mantle, and lifecycle decisions in Cairn.

#### Scenario: Roadmap preserves ownership and non-claims

- GIVEN an operator reads the consumer guide or AeneasVerif roadmap
- WHEN the integration path is described
- THEN the text MUST exclude Eurydice and Scylla from the consumer-contract implementation.
- AND it MUST disclaim translation soundness, model equivalence, verifier soundness, whole-program correctness, and release eligibility.

### Requirement: Final consumer validation is complete

r[octet.aeneas_consumer_contract.final_validation] The change MUST pass consumer profile, pure validation, bounded execution, artifact, compatibility, documentation, and Cairn validation before archive.

#### Scenario: Final consumer matrix is complete

- GIVEN implementation, fixtures, generated artifacts, and documentation are ready
- WHEN final validation runs
- THEN the positive consumer MUST pass and every negative fixture MUST fail for its expected reason.
- AND existing Aeneas, Verus, and starter paths MUST retain their scoped behavior.
