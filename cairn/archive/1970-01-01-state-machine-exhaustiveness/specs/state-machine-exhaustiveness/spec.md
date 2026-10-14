# State Machine Exhaustiveness Specification Delta

## ADDED Requirements

### Requirement: Marked transition-core contract
r[octet.state_machine_exhaustiveness.contract] Octet MUST define a marked transition-core contract for deliberately checked state machines over typed Rust state and event domains.

#### Scenario: Supported transition core is recognized
r[octet.state_machine_exhaustiveness.contract.supported]
- GIVEN a transition function is marked for state-machine checking and its typed inputs identify a state enum and an event enum
- WHEN Octet analyzes the function
- THEN Octet MUST classify the function as a state-machine transition core and record the state type, event type, variant inventories, and source span.

#### Scenario: Unsupported marked shape fails closed
r[octet.state_machine_exhaustiveness.contract.unsupported]
- GIVEN a marked transition function spreads decisions across mutable flags, runtime tags, trait-object dispatch, or macro-generated code whose enum domain cannot be inspected
- WHEN Octet analyzes the function
- THEN Octet MUST emit a deterministic unsupported-shape diagnostic instead of reporting the function as checked.

### Requirement: Explicit state/event matrix handling
r[octet.state_machine_exhaustiveness.matrix] Octet MUST require supported state-machine transition cores to handle every supported state/event pair through explicit variant patterns rather than catch-all fallback behavior.

#### Scenario: Explicit matrix passes
r[octet.state_machine_exhaustiveness.matrix.explicit]
- GIVEN a supported transition core explicitly matches every state/event pair and each pair returns either a next state or an invalid-transition error
- WHEN Octet runs the state-machine exhaustiveness check
- THEN the check MUST pass and record bounded source-shape evidence for that transition core.

#### Scenario: Wildcard fallback fails
r[octet.state_machine_exhaustiveness.matrix.wildcard]
- GIVEN a supported transition core uses `_`, rest, or catch-all fallback behavior to cover one or more state/event pairs
- WHEN Octet runs the state-machine exhaustiveness check
- THEN the check MUST fail with a diagnostic that asks for explicit state/event handling.

#### Scenario: Hidden invalid transition fails
r[octet.state_machine_exhaustiveness.matrix.hidden_invalid]
- GIVEN a transition core lets an invalid transition fall through a generic branch rather than returning an explicit invalid-transition result for that state/event pair
- WHEN Octet runs the state-machine exhaustiveness check
- THEN the check MUST fail with a deterministic hidden-invalid-transition diagnostic.

### Requirement: Pure transition decision boundary
r[octet.state_machine_exhaustiveness.pure_core] Octet MUST keep checked state-machine transition decisions in a pure functional core and reject mutation-coupled or ambient-effect transition decisions in the marked surface.

#### Scenario: Pure transition decision passes
r[octet.state_machine_exhaustiveness.pure_core.passes]
- GIVEN a marked transition core decides the next state from its typed inputs and returns a value without mutating caller-owned state or reading ambient resources
- WHEN Octet checks the transition core
- THEN the purity boundary MUST pass for the state-machine check.

#### Scenario: Mutation-coupled transition fails
r[octet.state_machine_exhaustiveness.pure_core.mutation]
- GIVEN a marked transition core mutates flags, shared state, or caller-owned state to encode the transition decision
- WHEN Octet checks the transition core
- THEN the check MUST fail with a diagnostic requiring an explicit returned transition decision.

### Requirement: State-machine fixture matrix
r[octet.state_machine_exhaustiveness.fixtures] Octet MUST include positive and negative fixtures for the supported state-machine exhaustiveness checker behavior.

#### Scenario: Positive fixtures cover explicit transitions
r[octet.state_machine_exhaustiveness.fixtures.positive]
- GIVEN a fixture declares a closed state/event enum pair and explicit valid and invalid transitions
- WHEN focused lint and UI tests run
- THEN the fixture MUST pass without state-machine exhaustiveness diagnostics.

#### Scenario: Negative fixtures cover fallback and unsupported shapes
r[octet.state_machine_exhaustiveness.fixtures.negative]
- GIVEN fixtures use wildcard fallback, hidden invalid transitions, unsupported generated shapes, or mutation-driven state changes
- WHEN focused lint and UI tests run
- THEN each fixture MUST fail closed with deterministic diagnostics.

### Requirement: Bounded receipt and Valence export boundary
r[octet.state_machine_exhaustiveness.receipts] Octet MUST emit state-machine exhaustiveness receipts as bounded source-shape evidence and MUST preserve explicit non-claims when exporting them to Valence.

#### Scenario: Receipt binds transition identity
r[octet.state_machine_exhaustiveness.receipts.identity]
- GIVEN a state-machine exhaustiveness check completes for a marked transition core
- WHEN Octet writes an artifact receipt or Valence export row
- THEN the receipt MUST bind checker identity, transition source span, state and event type identities, variant inventory digest, status, diagnostics, artifact BLAKE3 identity, and non-claims.

#### Scenario: Receipt does not become formal proof
r[octet.state_machine_exhaustiveness.receipts.no_promotion]
- GIVEN a state-machine exhaustiveness receipt is exported to Valence
- WHEN Octet validates the export graph
- THEN the receipt MUST remain bounded source-shape evidence and MUST NOT be promoted to formal proof without a separate Octet-owned proof receipt.

### Requirement: State-machine documentation
r[octet.state_machine_exhaustiveness.docs] Octet documentation MUST explain how consumers mark checked transition cores, how this check relates to enum-match lints, and which semantic claims remain out of scope.

#### Scenario: Boundary is visible to consumers
r[octet.state_machine_exhaustiveness.docs.boundary]
- GIVEN a consumer reads the state-machine exhaustiveness documentation
- WHEN the supported claim is stated
- THEN the documentation MUST say the check enforces explicit supported source shapes and does not prove whole-program correctness, liveness, protocol completeness outside the declared domain, or verifier soundness.

### Requirement: Final validation evidence
r[octet.state_machine_exhaustiveness.final_validation] The change MUST include focused lint tests, positive and negative UI fixtures, policy validation, artifact verification checks, and Cairn validation/gate evidence before archive.

#### Scenario: Validation covers checker health
r[octet.state_machine_exhaustiveness.final_validation.checks]
- GIVEN the implementation, fixtures, receipts, and documentation are complete
- WHEN final validation runs
- THEN focused tests, UI fixtures, policy validation, artifact checks, and Cairn gates MUST pass or record explicit pre-existing blockers.
