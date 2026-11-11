# Semantic Merge Verification Specification

## Purpose

Defines the `semantic-merge-verification` capability.

## Requirements

### Requirement: The profile names an exact bounded Rust subset

r[octet.semantic_merge_verification.profile]

Octet MUST define a versioned profile that names the admitted deterministic Rust
subset, its exact observables, preconditions, arithmetic and overflow behavior,
normal and abnormal outcomes, memory and aliasing model, dependency closure, and
explicit exclusions.

#### Scenario: Admitted request carries the reviewed assumptions
- GIVEN a request with the reviewed arithmetic, memory, dependency, precondition, external-effect, and observable-completeness assumptions
- WHEN admission runs
- THEN the assumptions MUST be bound into the admission identity and receipt
- AND the receipt MUST state the bounded claim scope.

#### Scenario: Construct outside the profile is rejected
- GIVEN a variant contains an unsafe, FFI, concurrency, macro, dispatch, or other unmodeled construct
- WHEN the checker evaluates the request
- THEN it MUST report `unsupported` for that construct
- AND it MUST NOT report `verified` or a semantic conflict.

#### Scenario: Vacuous precondition never verifies
- GIVEN an observable's precondition admits no input
- WHEN the checker determines coverage
- THEN it MUST report `unknown` with a vacuous-precondition reason
- AND it MUST NOT report a vacuous `verified`.

### Requirement: The relation is per-observable preservation

r[octet.semantic_merge_verification.relation]

Octet MUST check, for each declared observable and admitted input, that `M`
matches `A` when `A` changes the observation from `O`, matches `B` when `B`
changes it, and preserves `O` when neither changes it, and MUST report an
unsatisfiable relation when `A` and `B` change the same observation
incompatibly.

#### Scenario: Independent changes are preserved
- GIVEN `A` and `B` change different observables and `M` contains both changes
- WHEN the relation is checked
- THEN the outcome MUST be `verified` for the bounded domains.

#### Scenario: Incompatible changes are reported
- GIVEN `A` and `B` change one observable to different values
- WHEN the relation is checked
- THEN the outcome MUST be a `counterexample` with the incompatible class
- AND it MUST bind the O/A/B/M observed values and the input.

#### Scenario: A lost change is reported
- GIVEN `A` changes an observable and `M` matches `O` instead
- WHEN the relation is checked
- THEN the outcome MUST be a `counterexample` with the lost-A-change class.

#### Scenario: Text-clean interference is reported
- GIVEN the text merge is clean, neither variant changes an observable, and `M` changes it
- WHEN the relation is checked
- THEN the outcome MUST be a `counterexample` with the interference class.

### Requirement: Roles and mappings are tagged and validated

r[octet.semantic_merge_verification.identity]

Octet MUST bind role-tagged O/A/B/M source identities and semantic-subject
identities, MUST require exactly one variant and binding per role, and MUST
reject missing items, ambiguous mappings, wrong bindings, swapped roles, and
unreviewed signature or dependency changes as `unsupported` or as a structural
admission error.

#### Scenario: Renamed item has an explicit mapping
- GIVEN a logical item has different role-local names
- WHEN the request supplies one validated per-role mapping
- THEN resolution MUST use the logical identity for observables, calls, and receipts.

#### Scenario: Wrong or swapped binding fails closed
- GIVEN a declared source or subject identity does not match its role, or the role identities are a permutation
- WHEN binding validation runs
- THEN the outcome MUST be `unsupported` with a binding or role-mismatch reason
- AND the checker MUST NOT evaluate the relation as if the binding were valid.

#### Scenario: Missing observable fails closed
- GIVEN an observable is mapped but absent from one variant
- WHEN resolution runs
- THEN the outcome MUST be `unsupported` with a missing-observable reason.

#### Scenario: Changed dependency fails closed
- GIVEN a reachable callee that is not a declared observable has a different semantic identity in one variant
- WHEN dependency closure is validated
- THEN the outcome MUST be `unsupported` with a changed-dependency reason.

#### Scenario: Textual correspondence is insufficient
- GIVEN two variants have similar or identical source text but no validated source and subject bindings
- WHEN admission validates identities
- THEN textual similarity MUST NOT be accepted as correspondence evidence.

### Requirement: Outcomes are deterministic and distinct

r[octet.semantic_merge_verification.outcomes]

Octet MUST classify results as `verified`, `counterexample`, `unknown`,
`unsupported`, `timeout`, or `tool-error`, and MUST NOT treat a solver warning,
an exhausted budget, or a backend failure as a semantic conflict.

#### Scenario: Interrupted run is a timeout
- GIVEN an explicit evaluation or step budget is exhausted
- WHEN the checker stops the run
- THEN the outcome MUST be `timeout` with the interrupted phase and bound
- AND the outcome MUST NOT be `counterexample`.

#### Scenario: Backend failure is a tool error
- GIVEN the injected backend returns an error
- WHEN the checker runs
- THEN the outcome MUST be `tool-error` with the backend identity and message
- AND the outcome MUST NOT be a semantic conflict.

#### Scenario: Structural request failure issues no verdict
- GIVEN a missing or duplicate role or binding, a zero budget, or an empty observable set
- WHEN admission runs
- THEN Octet MUST return a deterministic admission error
- AND it MUST NOT issue a semantic verdict.

### Requirement: Receipts bind every checked fact and reject staleness

r[octet.semantic_merge_verification.receipts]

Octet MUST emit deterministic receipts binding the profile and claim scope,
assumptions, role-tagged identities, item mappings, observable domains,
dependency identities, frontend/model/verifier/backend/solver identities,
budgets, admission identity, exact command, deterministic work, outcome,
optional recorded-only Pijul provenance, and consumer non-claims, and MUST
reject stale or tampered receipts.

#### Scenario: Fresh receipt validates
- GIVEN a receipt and the matching current admission identity
- WHEN receipt validation runs
- THEN every payload identity, inventory, order, budget, work, non-claim, and outcome check MUST pass.

#### Scenario: Stale receipt is rejected
- GIVEN a receipt whose admission identity differs from the current request
- WHEN receipt validation runs
- THEN it MUST fail with a stale-input issue
- AND the receipt MUST NOT be reused as evidence for the changed request.

#### Scenario: Tampered payload is rejected
- GIVEN a receipt payload that no longer matches its receipt identity
- WHEN receipt validation runs
- THEN it MUST fail with an identity-mismatch issue.

#### Scenario: Pijul provenance stays recorded-only
- GIVEN a request supplies Pijul change identities
- WHEN the receipt is built
- THEN the identities MUST be recorded as optional provenance
- AND the checker MUST NOT treat clean application or equal trees as behavioral evidence.

### Requirement: Executed fixtures include independent expectations and negative controls

r[octet.semantic_merge_verification.fixtures]

Octet MUST provide executable fixtures for identical and independent changes,
equivalent edits with different syntax, incompatible changes, lost changes, and
text-clean semantic interference, together with negative controls for missing
observables, vacuous preconditions, changed callees, wrong bindings, swapped
roles, stale receipts, unsupported constructs, and interrupted verification.

#### Scenario: Corpus matches an independent expectation table
- GIVEN the reviewed fixture corpus
- WHEN the experiment runs
- THEN every case MUST be compared against an independently authored expectation
- AND the report MUST include case counts, expectation matches, precision, coverage, deterministic work, and unsupported or unknown counts.

#### Scenario: Negative controls do not verify
- GIVEN any negative-control fixture
- WHEN the checker runs
- THEN it MUST return its declared non-verified class
- AND it MUST NOT return `verified`.

### Requirement: External Lean backend records bounded concrete proofs

r[octet.semantic_merge_verification.external_backend]

Octet MUST provide an optional external Lean backend that, for a concrete
admitted input, asks the pinned Lean kernel to check a theorem binding the
encoded program result to the evaluator's observation. The backend MUST state
its trusted base, MUST reject arithmetic outside its declared profile, and MUST
classify Lean rejection, timeout, spawn failure, or nonzero exit as a tool
error, never a semantic conflict. Its evidence MUST be bounded to the supplied
input and MUST NOT be presented as a universal proof.

#### Scenario: Concrete evaluation is Lean-checked
- GIVEN a checked-arithmetic request and a concrete integer input
- WHEN the Lean backend evaluates the observable
- THEN Lean MUST accept the encoded concrete equality
- AND the receipt MUST record the Lean backend identity.

#### Scenario: Lean rejection or timeout is a tool error
- GIVEN Lean rejects the theorem, times out, or cannot run
- WHEN the backend returns
- THEN the checker MUST report `tool-error`
- AND it MUST NOT report a counterexample or semantic conflict.

#### Scenario: Wrapping is outside the backend profile
- GIVEN a request using wrapping arithmetic
- WHEN the Lean backend is selected
- THEN it MUST return `unsupported` for the backend profile
- AND it MUST NOT claim a Lean-checked result.

### Requirement: Bounded Rust frontend records syntactic correspondence

r[octet.semantic_merge_verification.frontend]

Octet MUST provide a bounded Rust source-to-IR frontend for the documented
scalar subset. The frontend MUST map every admitted function exactly once, MUST
fail closed on unsupported constructs, MUST bind the exact source text and
emitted program identities, and MUST emit an explicit correspondence report
with non-claims. The frontend MUST NOT claim semantic equivalence, compiler
correctness, or whole-program behavior.

#### Scenario: Admitted Rust source maps to the scalar IR
- GIVEN a source file in the documented `i64` scalar subset
- WHEN the frontend parses it
- THEN every admitted function MUST map exactly once
- AND the source and subject identities MUST bind the exact text and program.

#### Scenario: Unsupported Rust construct fails closed
- GIVEN a source file uses a construct outside the scalar subset
- WHEN the frontend parses it
- THEN it MUST return a typed unsupported error
- AND no partial program MUST be admitted.

### Requirement: Consumer contract preserves the bounded non-claims

r[octet.semantic_merge_verification.claim_boundary]

Octet MUST publish a go/no-go decision and consumer non-claims stating that the
rail checks only a supplied candidate over bounded domains, does not synthesize
or mutate a merge, does not prove Rust source-to-IR correspondence or
whole-program correctness, and does not grant release authority or provider
trust.

#### Scenario: Bounded prototype is authorized
- GIVEN the reviewed profile, prototype, fixtures, receipts, and non-claims
- WHEN the go/no-go decision is recorded
- THEN the bounded prototype MUST be a go
- AND production admission and universal verification MUST be a no-go.

#### Scenario: No downstream authority is implied
- GIVEN a consumer receives a `verified` receipt
- WHEN it interprets the result
- THEN it MUST preserve the receipt's claim scope and non-claims
- AND it MUST NOT treat the receipt as automatic merge admission, source mutation, whole-program correctness, or release authority.
