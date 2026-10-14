# standards-codification Specification

## Purpose

This spec defines the Tiger Style standards policy registry, standards-checker command surface, fixture expectations, suppression metadata, verification rails, and naming hygiene needed to keep codified standards traceable to their source material.
## Requirements
### Requirement: Tiger Style Policy Registry

The system MUST maintain a machine-readable Tiger Style policy registry that traces each codification candidate to `docs/standards/source/tigerstyle-standards.adoc` or an included `docs/standards/source/*.adoc` source section.

Each policy entry MUST include a stable `tigerstyle.*` policy ID, pillar, source path, source heading, summary, enforcement level, checker owner, default severity, suppression requirements, and verification evidence requirements.

#### Scenario: Codifiable source becomes policy

- GIVEN a Tiger Style source section states a repeatable rule such as "every unbounded queue is a latent OOM"
- WHEN the policy inventory is generated or updated
- THEN the registry contains a policy entry linked to `docs/standards/source/embedded.adoc`
- AND the entry classifies whether the rule is hard, advisory, or human-process-only
- AND the entry names the checker or planned checker that owns enforcement

#### Scenario: Human-only guidance stays explicit

- GIVEN a Tiger Style source section describes flow protection, regulatory classification, or Five Whys facilitation
- WHEN the policy inventory is reviewed
- THEN the registry marks the entry as human-process-only or advisory
- AND no blocking build rule is created without a separate, measurable enforcement design

#### Scenario: Missing traceability is rejected

- GIVEN a policy entry lacks a source path or source heading
- WHEN the registry is validated
- THEN validation fails with a diagnostic naming the incomplete policy entry

### Requirement: Typed Policy Configuration

The system MUST use typed Nickel contracts for project-owned Tiger Style policy configuration, with generated machine formats used only as tool inputs.

The policy schema MUST reject entries with missing required fields, unknown enforcement levels, missing suppression metadata, or checker references that do not map to a known checker.

#### Scenario: Valid Nickel policy exports for tools

- GIVEN a policy registry that satisfies the Nickel contract
- WHEN the export command runs
- THEN it produces deterministic tool input data
- AND the exported data preserves policy IDs, source links, enforcement levels, and checker ownership

#### Scenario: Invalid policy fails before checks run

- GIVEN a policy entry declares an unknown enforcement level
- WHEN registry validation runs
- THEN the command fails before running source-code checks
- AND the diagnostic identifies the invalid field and policy ID

### Requirement: Reusable Tiger Style Standards Runner

The system MUST provide a reusable `tigerstyle-standards` checker command with a pure rule-evaluation core and a thin imperative shell for filesystem traversal, CLI parsing, and process exit codes.

The runner MUST support selecting rule families, validating fixtures, emitting stable human-readable diagnostics, and producing machine-readable output for CI dashboards.

#### Scenario: Blocking hard rule fails deterministically

- GIVEN a fixture that violates a hard policy
- WHEN the checker runs against that fixture
- THEN the command exits non-zero
- AND the diagnostic includes policy ID, file path, line when available, source section, and remediation summary

#### Scenario: Advisory rule reports without blocking

- GIVEN a fixture that only violates advisory policies
- WHEN the checker runs in default CI mode
- THEN the command exits successfully
- AND the report includes the advisory finding with its policy ID and source section

### Requirement: Hard Rule Families

The system MUST codify high-confidence Tiger Style rules as hard-rule families when the rule can be checked with low false-positive risk and clear remediation.

At minimum, the policy plan MUST cover these hard-rule candidates:

- Architecture: FCIS crate boundary, no external domain type leakage into core, explicit conversion at boundaries, root crate thinness, and controlled public API surface.
- DevTools: single-command verification, deny warnings, formatting enforcement, documentation gates, dependency audit gates, and no unexplained lint suppressions.
- Embedded: bounded channels and queues, timeout-backed blocking I/O, opt-in subsystem startup from a hardware manifest, safe serial APIs, and bounded task/thread topology.
- Resilience: explicit typed errors, no silent defaulting on fallible paths, modeled failure modes instead of catch-all buckets, and fail-closed error handling.
- Orchestration: declarative transition tables, deny-by-default work acceptance, invalid transitions that leave state unchanged, and priority ordering properties.
- Idioms: one behavior / one type extraction candidates, duplicate primitive-behavior detection, and verifier recognition of approved thin-forwarder idiom output.
- Build What You Test: explicit test builders for broad config structs, visible preconditions, no hidden external fixtures in unit tests, and isolated process-global state.
- Metrics: flat telemetry payloads, threshold-backed health checks, Prometheus text exposition when metrics endpoints exist, and BLAKE3 hash-chain audit integrity.
- Verification: positive and negative tests, FIRST properties where statically checkable, target/hardware coverage declarations, and safety-critical evidence prompts.

#### Scenario: Candidate rule with weak signal becomes advisory

- GIVEN a candidate rule cannot distinguish safe code from unsafe code without human context
- WHEN the hard-rule family is reviewed
- THEN the registry classifies it as advisory or human-process-only
- AND the design records why it is not a blocking gate yet

#### Scenario: Hard rule has fixture evidence

- GIVEN a hard-rule policy is enabled
- WHEN fixture validation runs
- THEN at least one passing fixture proves the checker accepts the intended shape
- AND at least one failing fixture proves the checker rejects the unsafe or noncompliant shape

### Requirement: Suppressions and Escape Hatches

The system MUST make any suppression of a Tiger Style hard rule explicit, traceable, and narrow.

Suppressions MUST use `tigerstyle-allow:` and include policy ID, source section, justification, owner, review trigger, and a reference to the safer default being bypassed. Blanket allow-lists that hide future true positives MUST be rejected.

#### Scenario: Narrow suppression is accepted

- GIVEN a hardware-bound module needs a justified exception to a hard rule
- WHEN the suppression includes policy ID, source section, owner, rationale, review trigger, and the safer default being bypassed
- THEN the checker accepts only the annotated location
- AND the report records the active suppression

#### Scenario: Blanket suppression is rejected

- GIVEN a suppression disables a rule across a module or workspace without policy ID and owner
- WHEN the checker runs
- THEN the command fails
- AND the diagnostic points to the suppression rule and remediation format

### Requirement: Verification Rail Integration

The system MUST integrate Tiger Style standards codification with existing verification rails through local-first commands and flake checks.

The command set MUST remain runnable from the Nix dev shell and MUST include standards checker self-tests, policy registry validation, fixture validation, workspace checks, Clippy denial, and Tiger Style lint checks as separate seams.

#### Scenario: Local-first standards verification succeeds

- GIVEN a developer enters the Nix dev shell or invokes `nix develop path:. -c ...`
- WHEN the documented Tiger Style standards verification commands run
- THEN they execute standards self-tests, policy validation, fixture checks, formatting, and workspace checks with repo-local target directories where Cargo artifacts are involved
- AND they exit successfully for a compliant workspace

#### Scenario: Fixture regression blocks CI

- GIVEN a change weakens a checker so a failing fixture is no longer reported
- WHEN fixture validation runs under the flake check
- THEN CI fails with the fixture name, expected policy ID, and missing diagnostic

### Requirement: Rename Hygiene

The system MUST remove legacy project naming from Tiger Style-owned standards artifacts.

The migrated standards source, policy IDs, suppression marker, checker command, docs, Nix apps, and OpenSpec artifacts MUST use Tiger Style naming.

#### Scenario: Legacy project names are absent from Tiger Style standards artifacts

- GIVEN the standards migration is staged in the `tigerstyle` repo
- WHEN a case-insensitive source scan runs over Rust, Markdown, Nickel, AsciiDoc, Nix, TOML, YAML, and text artifacts
- THEN it finds no legacy project strings strings in the migrated Tiger Style standards surface

### Requirement: External Source Incorporation Classification [r[external-source-incorporation-classification]]

The system MUST classify each incorporated external reference concept before it becomes Tigerstyle standards guidance, policy registry data, or lint scope.

Each classification MUST name the external source, the imported concept, the Tiger Style outcome it supports, the enforcement tier, and whether the first implementation is documentation, policy metadata, a checker, or a human checklist.

#### Scenario: External concepts are classified before incorporation [r[external-source-incorporation-classification.scenario.classified-before-incorporation]]

- GIVEN a contributor proposes importing a Valgrind tool-suite concept or a Rust Design Patterns idiom
- WHEN the standards incorporation plan is updated
- THEN the plan records the source URL, selected concept, Tiger Style rationale, and enforcement tier
- AND concepts with weak static signal remain advisory or human-process-only until separate fixture evidence exists

### Requirement: Dynamic Evidence Profiles [r[dynamic-evidence-profiles]]

The system MUST document opt-in dynamic evidence profiles for bug classes that static Tigerstyle lints cannot fully prove.

The profiles MUST cover memory/allocation behavior, cache or hot-path profiling, race/concurrency analysis, unsafe or FFI boundary checking, and release-readiness timing. Each profile MUST state when to run it, the expected acceptance signal, and how evidence is saved for OpenSpec review.

#### Scenario: Dynamic profile guidance names run timing and acceptance signal [r[dynamic-evidence-profiles.scenario.timing-and-acceptance]]

- GIVEN a maintainer selects an unsafe, FFI, concurrency, or performance-sensitive slice
- WHEN the relevant dynamic evidence profile is read
- THEN the profile names the applicable run timing such as after a big change, suspected bug, pre-release, or focused CI rail
- AND it names the acceptance signal or saved transcript shape required for review

#### Scenario: Heavyweight tools remain optional until a profile is selected [r[dynamic-evidence-profiles.scenario.optional-until-selected]]

- GIVEN a normal Tigerstyle consumer runs the default lint or standards checks
- WHEN no dynamic evidence profile is selected
- THEN the default command set does not require Valgrind, sanitizers, heap profilers, cache profilers, Miri, or Loom to be installed
- AND profile docs explain how to run selected tools only for applicable evidence slices

### Requirement: Rust Idiom Guidance [r[rust-idiom-guidance]]

The system MUST provide Tigerstyle-owned Rust idiom guidance that imports only patterns that improve safety, explicitness, boundedness, recoverability, performance evidence, or developer comprehension.

The guidance MUST cover newtypes for units/capabilities/secrets, RAII resource guards, contained unsafe modules, borrowed argument types, temporary mutability, clone-to-satisfy-borrow-checker alternatives, returning consumed arguments on error, constrained builders, command/effects-as-data, FFI wrappers and error mapping, restrained `Deref` use, small-crate boundaries, and functional API patterns that use generics or sealed traits to make invalid protocol/state operations unrepresentable.

#### Scenario: Rust idiom guidance maps each imported pattern to Tiger Style outcomes [r[rust-idiom-guidance.scenario.maps-patterns-to-outcomes]]

- GIVEN a reader opens the Rust idiom guidance
- WHEN they review an imported pattern or anti-pattern
- THEN the guidance states the Tiger Style outcome it supports
- AND it distinguishes immediate documentation/checklist guidance from future lint or policy enforcement

#### Scenario: Future lints require fixtures before hard gating [r[rust-idiom-guidance.scenario.future-lints-require-fixtures]]

- GIVEN a pattern-derived lint candidate such as borrowed argument types, unsafe module containment, clone-avoidance, temporary mutability, or `Deref` restraint is proposed
- WHEN the policy registry classifies it as a hard rule
- THEN positive and negative fixtures prove the intended compliant and noncompliant shapes
- AND the lint is not promoted to a default hard gate without that fixture evidence

### Requirement: Functional API Boundary Guidance [r[functional-api-boundary-guidance]]

The system MUST document which functional programming concepts from Rust Design Patterns are acceptable Tigerstyle guidance and which remain non-goals.

The guidance MUST promote type-class-style generics, sealed traits, associated types, and marker-state APIs when they encode protocol, state, capability, or format differences at compile time. It MUST treat optics, visitors, and prism-like adapter splits as advanced human-review guidance for serialization, traversal, and plugin boundaries rather than default application style. It MUST caution against opaque point-free or deeply chained functional code in hot paths, safety-critical paths, and code that needs dense assertions.

#### Scenario: Functional API guidance promotes type-level state separation without opaque cleverness [r[functional-api-boundary-guidance.scenario.type-level-state-not-cleverness]]

- GIVEN a Tigerstyle guide imports functional programming concepts
- WHEN a reader compares type-class/generic-state APIs, optics/visitor adapter APIs, and fold-heavy declarative code
- THEN the guide recommends type-level state or protocol separation where it removes runtime `Option`/enum checks from known-state code paths
- AND it classifies optics or visitor/prism patterns as advanced boundary tools requiring human design review
- AND it warns that long iterator chains or point-free transformations are inappropriate when they hide bounds, branches, assertions, or hot-loop costs

### Requirement: Function Fingerprint Policy [r[function-fingerprint-policy]]

The system MUST define a Tigerstyle policy for selective content-addressed function fingerprints that bind safety-critical Rust functions to exact reviewed or verified source-token bodies.

The policy MUST state that fingerprints are identity and review gates, not correctness proofs. It MUST recommend fingerprints for verified primitives, protocol transitions, security-sensitive functions, capability/token constructors, safety-critical arithmetic, and fixture-backed documentation examples. It MUST NOT require blanket fingerprints for every function in a workspace.

#### Scenario: Critical function categories are documented [r[function-fingerprint-policy.scenario.critical-categories]]

- GIVEN a maintainer reads the function fingerprint guidance
- WHEN they decide whether a function should be fingerprinted
- THEN the guidance names verified primitives, protocol transitions, security-sensitive functions, capability/token constructors, safety-critical arithmetic, and fixture-backed examples as recommended categories
- AND the guidance says ordinary functions do not need fingerprints by default

#### Scenario: Fingerprints do not claim correctness [r[function-fingerprint-policy.scenario.identity-not-correctness]]

- GIVEN a fingerprinted function matches its stored hash
- WHEN the tool reports the fingerprint as valid
- THEN the documentation and diagnostic language state that the exact source body matches the reviewed body
- AND they do not claim the function is semantically correct, safe, or verified solely because the hash matches

### Requirement: Function Fingerprint Metadata [r[function-fingerprint-metadata]]

The system MUST require review metadata for function fingerprints so each selected function records why content identity matters.

At minimum, each fingerprint MUST name the hash value and review class. Supported review classes MUST include `verified`, `protocol-transition`, `security`, `capability-token`, `safety-arithmetic`, and `example-evidence`. Optional metadata MAY include owner, spec ID, evidence path, and macro allowance.

#### Scenario: Missing review class is rejected [r[function-fingerprint-metadata.scenario.missing-review-class]]

- GIVEN a function carries a fingerprint hash but omits review class metadata
- WHEN fingerprint validation runs in strict mode
- THEN validation reports the function as incomplete
- AND the diagnostic names the missing review class field

#### Scenario: Review metadata links to evidence [r[function-fingerprint-metadata.scenario.links-to-evidence]]

- GIVEN a fingerprinted verified primitive or protocol transition names a spec ID or evidence path
- WHEN fingerprint inventory output is generated
- THEN the output preserves the function name, hash, review class, and evidence reference
- AND reviewers can trace the function identity to the relevant OpenSpec or verification artifact

### Requirement: Function Fingerprint Hash Semantics [r[function-fingerprint-hash-semantics]]

The system MUST compute function fingerprints from a versioned, domain-separated, normalized representation of Rust function source tokens.

The hash input MUST include normalized visibility, signature, generic parameters, where clauses, and body tokens. It MUST exclude the fingerprint attribute or marker itself, whitespace, comments, file paths, line numbers, timestamps, local usernames, and absolute paths. The hash algorithm MUST be identified with a `b3:` prefix for BLAKE3 values.

#### Scenario: Formatting and comments do not change hash [r[function-fingerprint-hash-semantics.scenario.formatting-stable]]

- GIVEN a fingerprinted function whose executable tokens are unchanged
- WHEN only rustfmt layout, comments, or line numbers change
- THEN recomputing the fingerprint yields the same `b3:` value

#### Scenario: Signature or body token changes update hash [r[function-fingerprint-hash-semantics.scenario.token-change-detected]]

- GIVEN a fingerprinted function with a stored hash
- WHEN its signature, generics, where clause, visibility, or body tokens change
- THEN fingerprint validation reports a mismatch with expected and actual `b3:` values

### Requirement: Function Fingerprint Macro Boundary [r[function-fingerprint-macro-boundary]]

The system MUST make macro-call limitations explicit for fingerprinted functions.

If a fingerprinted function contains source macro invocations, validation MUST report a macro-boundary warning unless the fingerprint metadata explicitly allows macros. The warning MUST explain that source-token fingerprints do not cover macro definition changes or expansion semantics.

#### Scenario: Macro call warns by default [r[function-fingerprint-macro-boundary.scenario.warns-by-default]]

- GIVEN a fingerprinted function body contains a macro invocation
- WHEN fingerprint validation runs
- THEN validation emits a macro-boundary diagnostic
- AND the diagnostic explains that macro definitions are not covered by the function source-token hash

#### Scenario: Explicit macro allowance suppresses caveat [r[function-fingerprint-macro-boundary.scenario.explicit-allowance]]

- GIVEN a fingerprinted function body contains a macro invocation and metadata explicitly allows macros
- WHEN fingerprint validation runs
- THEN validation does not emit the default macro-boundary warning
- AND inventory output preserves that macro allowance for review

### Requirement: Effect Capability Boundary Guidance [r[effect-capability-boundary-guidance]]

The system MUST document a Rust effect-capability boundary model that keeps domain core logic free of ambient IO, time, randomness, env access, process spawning, networking, and remote execution while allowing effects to appear as explicit data or named capability traits.

#### Scenario: Core effects are explicit [r[effect-capability-boundary-guidance-scenario-core-effects-are-explicit]]

- GIVEN a developer designs a core function
- WHEN they consult the guidance
- THEN it describes returning effect descriptions or accepting named capabilities instead of reading ambient state directly

#### Scenario: Shell interprets capabilities [r[effect-capability-boundary-guidance-scenario-shell-interprets-capabilities]]

- GIVEN core logic emits an effect description
- WHEN shell code executes the program
- THEN the shell owns IO, time, randomness, network, process, and remote side effects

### Requirement: Future effect lint classification [r[future-effect-lint-classification]]

The system MUST classify future effect-boundary lint candidates before hard gating them, including effectful globals, effectful constructors, and expanded pure-module checks.

#### Scenario: Lint candidates stay staged [r[future-effect-lint-classification-scenario-lint-candidates-stay-staged]]

- GIVEN an effect-boundary checker has weak static signal
- WHEN policy metadata is reviewed
- THEN the checker remains advisory or human-process-only until positive and negative fixtures prove low false-positive behavior

### Requirement: Top Level Purity Guidance [r[top-level-purity-guidance]]

The system MUST document that Rust globals, static initializers, lazy initializers, constructors, Default implementations, and builders should not hide filesystem, env, clock, randomness, network, process, or spawn effects.

#### Scenario: Initializer side effects are rejected by guidance [r[top-level-purity-guidance-scenario-initializer-side-effects-are-rejected-by-guidance]]

- GIVEN a LazyLock initializer reads env or files
- WHEN a reviewer applies the guidance
- THEN it is classified as shell work or delayed effect construction rather than pure initialization

#### Scenario: Constructors validate only [r[top-level-purity-guidance-scenario-constructors-validate-only]]

- GIVEN a `new`, `default`, or `build` function exists in core logic
- WHEN it is reviewed
- THEN the accepted shape validates and constructs values without performing ambient effects

### Requirement: Delayed Effect Pattern [r[delayed-effect-pattern]]

The system MUST document delayed-effect alternatives such as returning commands, plans, tasks, or capability requests for shell interpretation.

#### Scenario: Effect is represented as data [r[delayed-effect-pattern-scenario-effect-is-represented-as-data]]

- GIVEN construction needs future IO
- WHEN the code follows the delayed-effect pattern
- THEN it returns an explicit effect description and leaves execution to shell code

### Requirement: Remote Execution Effect Contract [r[remote-execution-effect-contract]]

The system MUST document remote execution as a first-class effect family that requires explicit timeout, retry bound, batch bound, idempotency key, and receipt or replay semantics.

#### Scenario: Remote task carries bounds [r[remote-execution-effect-contract-scenario-remote-task-carries-bounds]]

- GIVEN a design schedules remote work
- WHEN the remote effect contract is applied
- THEN the task model names timeout, max retries, idempotency key, and batch bound

#### Scenario: Receipt semantics are explicit [r[remote-execution-effect-contract-scenario-receipt-semantics-are-explicit]]

- GIVEN remote work completes or fails
- WHEN review evidence is captured
- THEN the receipt records enough identity to replay, correlate, or audit the attempted work

### Requirement: Remote Lint Staging [r[remote-lint-staging]]

The system MUST keep remote-effect enforcement advisory or checklist-based until fixtures prove a low false-positive static signal.

#### Scenario: Weak static signal remains human reviewed [r[remote-lint-staging-scenario-weak-static-signal-remains-human-reviewed]]

- GIVEN remote execution APIs differ by consumer
- WHEN a checker is proposed
- THEN the policy records human-process or advisory classification before hard gating

### Requirement: Named Capability Vocabulary [r[named-capability-vocabulary]]

The system MUST document a Rust capability vocabulary pattern for domain effects using explicit traits, tokens, or effect enums with names that expose authority and lifecycle obligations.

#### Scenario: Capability name exposes authority [r[named-capability-vocabulary-scenario-capability-name-exposes-authority]]

- GIVEN code needs clock, entropy, credential, object-store, or remote authority
- WHEN the capability is modeled
- THEN its name and API expose the authority being granted rather than hiding it behind generic context objects

#### Scenario: Core admissibility is explicit [r[named-capability-vocabulary-scenario-core-admissibility-is-explicit]]

- GIVEN a capability crosses into core-adjacent logic
- WHEN it is reviewed
- THEN guidance distinguishes pure data capabilities from shell-only effectful capabilities

### Requirement: Deterministic Capability Handlers [r[deterministic-capability-handlers]]

The system MUST require examples for capability traits to include deterministic test handlers such as fixed clocks, seeded entropy, recording stores, or fake remote executors.

#### Scenario: Test handler avoids ambient state [r[deterministic-capability-handlers-scenario-test-handler-avoids-ambient-state]]

- GIVEN a capability example appears in docs
- WHEN the test form is shown
- THEN it uses deterministic handler state rather than ambient time, randomness, network, or filesystem

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

### Requirement: Primitive Alias Lint Staging [r[primitive-alias-lint-staging]]

The system MUST classify future primitive-alias or missing-newtype lint candidates as staged until fixtures prove that warnings identify safety-relevant ambiguity.

#### Scenario: Primitive lint needs evidence [r[primitive-alias-lint-staging-scenario-primitive-lint-needs-evidence]]

- GIVEN a future checker flags raw IDs or unit values
- WHEN hard-gate promotion is considered
- THEN positive and negative fixtures prove it avoids harmless local primitives

### Requirement: Deterministic Handler Test Guidance [r[deterministic-handler-test-guidance]]

The system MUST document deterministic handler testing for effectful boundaries, including fixed clocks, seeded randomness, recording stores, fake remotes, and configured failure handlers.

#### Scenario: Handler test is ambient-free [r[deterministic-handler-test-guidance-scenario-handler-test-is-ambient-free]]

- GIVEN a test exercises effect-boundary logic
- WHEN deterministic handler guidance is applied
- THEN the test uses injected handler state rather than live time, randomness, network, or filesystem

#### Scenario: Recording handler proves requested effects [r[deterministic-handler-test-guidance-scenario-recording-handler-proves-requested-effects]]

- GIVEN core code emits effect descriptions
- WHEN a recording handler test runs
- THEN the test can assert the exact ordered effects without performing real side effects

### Requirement: Invalid Path Handler Evidence [r[invalid-path-handler-evidence]]

The system MUST require handler tests to cover invalid paths and cleanup obligations where the effect contract includes failure behavior.

#### Scenario: Failure handler proves cleanup [r[invalid-path-handler-evidence-scenario-failure-handler-proves-cleanup]]

- GIVEN an effect handler can fail after partial progress
- WHEN invalid-path evidence is captured
- THEN the evidence shows the reported error and cleanup or no-mutation behavior

### Requirement: Policy Enforcement Mode Taxonomy [r[policy-enforcement-mode-taxonomy]]

The system MUST let each Tiger Style policy declare a current enforcement mode and target enforcement mode chosen from `lint`, `gate`, `structural`, `verified`, and `human-oracle`.

Each mode declaration MUST name the checker or mechanism responsible for enforcement, the escalation criteria for moving to a stronger mode, and the accepted fallback when a stronger mode is not yet possible.

#### Scenario: Lint-only policy remains explicit [r[policy-enforcement-mode-taxonomy.scenario.lint-only-explicit]]

- GIVEN a Tiger Style policy has useful signal but no reliable structural Rust mechanism
- WHEN the policy registry is validated
- THEN the policy remains valid only if it declares `current = "lint"` or `current = "human-oracle"`
- AND the entry names why structural or verified enforcement is not currently required

#### Scenario: Target mode stronger than current mode records escalation [r[policy-enforcement-mode-taxonomy.scenario.target-escalation-recorded]]

- GIVEN a policy currently gates diagnostics but should eventually become construction-enforced
- WHEN the policy registry is exported
- THEN the exported policy includes both current and target enforcement modes
- AND it includes the fixture or proof evidence required before promotion

### Requirement: Nickel Typed Policy Contracts [r[nickel-typed-policy-contracts]]

The system MUST define Tiger Style policy configuration in Nickel with typed, closed contracts for policies, enforcement modes, mechanisms, evidence, suppressions, proof rails, and reference seed manifests.

The Nickel source MUST be the authoring surface for policy intent and MUST export deterministic machine data consumed by Rust checkers; Rust checkers MUST NOT treat ad-hoc TOML, JSON, or CLI flags as the source of policy truth when a Nickel policy export exists.

#### Scenario: Closed Nickel policy contract rejects unknown fields [r[nickel-typed-policy-contracts.scenario.closed-contract-rejects-unknown-fields]]

- GIVEN a policy entry contains an undeclared field or misspelled enforcement mode
- WHEN Nickel policy validation runs
- THEN validation fails before source-code checkers run
- AND the diagnostic names the invalid policy entry and field

#### Scenario: Structural mode requires structural mechanism contract [r[nickel-typed-policy-contracts.scenario.structural-requires-mechanism]]

- GIVEN a policy declares `current = 'structural`
- WHEN Nickel cross-field validation evaluates the policy
- THEN the policy is valid only if `mechanism.kind` is one of `crate_boundary`, `capability_api`, `bounded_primitive`, `typestate`, or `seed_corpus`
- AND the required mechanism-specific contract fields are present

#### Scenario: Verified mode requires proof rail contract [r[nickel-typed-policy-contracts.scenario.verified-requires-proof-rail]]

- GIVEN a policy declares `current = 'verified`
- WHEN Nickel cross-field validation evaluates the policy
- THEN the policy is valid only if `mechanism.kind = 'proof_rail`
- AND the proof backend, command, source paths, expected signal, and trust-boundary accounting are present

#### Scenario: Deny severity requires deterministic checker and evidence [r[nickel-typed-policy-contracts.scenario.deny-requires-checker-evidence]]

- GIVEN a policy declares `severity = 'deny`
- WHEN Nickel cross-field validation evaluates the policy
- THEN the policy is valid only if a deterministic checker command and expected evidence signal are configured
- AND suppressions, when allowed, require policy ID, source section, owner, rationale, review trigger, and safer default metadata

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

- GIVEN Aspen pins a Tigerstyle flake exposing `lib.mkPolicyArtifactConsumerCheck` and checks selected embedded roots with `checks.x86_64-linux.octet-policy-artifacts`
- WHEN a scratch Aspen copy inserts exactly one configured unbounded queue constructor inside a checked root
- THEN the focused Aspen policy artifact check exits non-zero
- AND the saved transcript names `tigerstyle.embedded.bounded-queues`, the mutated file, and the single expected mutation diagnostic before the scratch mutation is removed or discarded

#### Scenario: Focused Tigerstyle checks pass [r[structural-mechanism-validation-scenario-focused-typestate-checks-pass]]

- GIVEN the type-state structural checker, fixtures, policy registry, and generated artifact export are changed together
- WHEN the focused Rust, Nickel/Nix, OpenSpec, docs, and whitespace checks run
- THEN every focused check exits successfully and the saved transcript names the commands used

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]
Verified proof rails MUST bind proof receipts to the configured verifier identity, toolchain identity, executable identity, build closure identity, and Nix closure graph evidence.

#### Scenario: Receipt records generated Nix closure graph digest [r[verified-enforcement-rails.nix-closure-graph-digest]]

- GIVEN a verified proof rail configured with `verifier_nix_closure_graph_paths` and `verifier_nix_closure_graph_digest`
- WHEN the checker validates a passing proof receipt
- THEN the checker MUST recompute a BLAKE3 digest from the configured generated Nix `exportReferencesGraph` artifacts
- AND the checker MUST require a matching `proof-verifier-nix-closure-graph-blake3` receipt line

#### Scenario: Generated artifacts expose Nix closure graph obligations [r[verified-enforcement-rails.nix-closure-graph-artifacts]]

- GIVEN policy artifacts are exported for a verified proof rail with Nix closure graph obligations
- WHEN downstream checkers consume the generated artifact bundle
- THEN the artifact bundle MUST include the configured Nix closure graph digest and graph artifact path obligations

### Requirement: Reference Seed Corpus Vendoring [r[reference-seed-corpus-vendoring]]

The system MUST support optional vendoring of reference code, fixtures, examples, or proof seeds from `../vericoder-mvp/`, `../verified-logic/`, and `../ucan/` when those seeds help implement structural or verified Tiger Style enforcement.

Vendored seeds MUST include a manifest recording source repository, source commit, selected paths, BLAKE3 or SHA-256 hashes, license/provenance review, intended role, local acceptance command, and whether the seed is advisory, fixture-only, reusable library code, or proof-relevant.

#### Scenario: Vericoder seed is accepted as architecture or trajectory fixture [r[reference-seed-corpus-vendoring.scenario.vericoder-seed-fixture]]

- GIVEN a future implementation imports Vericoder MVP core/shell patterns, trajectories, or analyzer fixtures
- WHEN the seed manifest is validated
- THEN the seed is accepted only if it records the source commit, path list, hashes, and local Tigerstyle acceptance command
- AND the manifest marks the seed as advisory architecture/fixture evidence rather than proof of Tigerstyle correctness

#### Scenario: Verified-logic seed requires local proof boundary [r[reference-seed-corpus-vendoring.scenario.verified-logic-proof-boundary]]

- GIVEN a future implementation imports proof-relevant primitives from verified-logic
- WHEN the seed is used for a verified Tiger Style policy
- THEN Tigerstyle validation requires local proof or a recorded trust-boundary contract in this repository
- AND sibling repository proof status alone is not accepted as the final evidence artifact

#### Scenario: UCAN seed demonstrates core-shell vendoring shape [r[reference-seed-corpus-vendoring.scenario.ucan-core-shell-seed]]

- GIVEN a future implementation imports UCAN examples of a no-std core, std shell, or vendored verified-logic bridge
- WHEN structural validation runs against the seed fixture
- THEN the fixture proves the checker accepts the intended core/shell dependency direction and vendored-revision manifest shape
- AND a negative fixture proves ambient effects in the core still fail

### Requirement: Enforcement Mode Output and CI Behavior [r[enforcement-mode-output-ci-behavior]]

The system MUST expose enforcement-mode validation results through deterministic human output, machine output, and local-first CI or Nix gates.

Reports MUST distinguish lint findings, gate failures, structural-contract failures, verified-proof failures, accepted suppressions, and advisory/human-oracle reminders.

#### Scenario: Machine output classifies enforcement failure type [r[enforcement-mode-output-ci-behavior.scenario.machine-output-classifies-failure]]

- GIVEN a workspace violates one lint-only policy and one structural policy
- WHEN the checker emits machine-readable output
- THEN each finding includes policy ID, enforcement mode, failure kind, configured mechanism, source location when available, and remediation summary
- AND structural-contract failures are distinguishable from ordinary lint diagnostics

#### Scenario: Nix or CI gate fails on missing structural evidence [r[enforcement-mode-output-ci-behavior.scenario.ci-fails-missing-structural-evidence]]

- GIVEN a policy is configured as structurally enforced
- WHEN the local-first CI or Nix check runs without the required crate/API/fixture evidence
- THEN the check exits non-zero
- AND the diagnostic names the missing structural mechanism rather than reporting a generic lint failure

### Requirement: Generated Policy Artifacts [r[generated-policy-artifacts]]

The system MUST export deterministic generated artifacts from the validated Tiger Style policy registry for consumer checkers and CI review.

The artifact bundle MUST include `octet-policies.json`, `octet-check-config.toml`, and `octet-seed-manifests.json`.

#### Scenario: Artifact export writes named files [r[generated-policy-artifacts.scenario.named-files]]

- GIVEN a registry JSON exported from `standards/policies/registry.ncl`
- WHEN the standards artifact export command writes to an output directory
- THEN the output directory contains `octet-policies.json`
- AND the output directory contains `octet-check-config.toml`
- AND the output directory contains `octet-seed-manifests.json`

#### Scenario: Artifact-backed checker run [r[generated-policy-artifacts.scenario.checker-consumes-artifacts]]

- GIVEN a generated artifact directory
- WHEN `tigerstyle-standards check` runs with that directory and a structural fixture
- THEN the checker loads the policy registry from `octet-policies.json`
- AND the checker emits the same policy diagnostics and enforcement metadata as the registry-backed path

#### Scenario: Check config summarizes enforcement seams [r[generated-policy-artifacts.scenario.check-config-summary]]

- GIVEN generated policy artifacts
- WHEN the check config artifact is inspected
- THEN it lists checker IDs, policy families, hard policy IDs, structural policy IDs, and verified policy IDs in deterministic order

### Requirement: Artifact-backed effect capability mechanism obligations [r[artifact-backed-effect-capability-mechanism-obligations]]

The system MUST let the `tigerstyle.architecture.effect-capability-boundaries` policy export capability-api mechanism obligations through generated policy artifacts.

#### Scenario: Capability obligations are exported [r[artifact-backed-effect-capability-mechanism-obligations.scenario.exported]]

- GIVEN the Nickel registry declares a capability-api mechanism for effect capability boundaries
- WHEN generated policy artifacts are exported
- THEN the `tigerstyle.architecture.effect-capability-boundaries` artifact entry carries forbidden ambient APIs, core source scopes, shell constructor scopes, and approved capability names
- AND the artifact-backed checker can consume those fields without reading Nickel source directly

### Requirement: Core ambient authority structural rejection [r[core-ambient-authority-structural-rejection]]

The system MUST reject declared core source code that performs configured ambient authority operations instead of accepting injected data or narrow capability traits.

#### Scenario: Core ambient operation fails [r[core-ambient-authority-structural-rejection.scenario.core-ambient-fails]]

- GIVEN a Tigerstyle architecture fixture whose declared core scope calls a configured ambient authority API such as `std::fs`, `std::env`, `std::net`, `SystemTime::now`, or process spawning
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs for the architecture family
- THEN the report contains a structural diagnostic for `tigerstyle.architecture.effect-capability-boundaries`
- AND the diagnostic remediation tells the developer to move ambient constructors to the shell and inject data or named capabilities into core

### Requirement: Injected capability and shell construction acceptance [r[injected-capability-shell-construction-acceptance]]

The system MUST accept declared core source code that represents effects as data or named capability traits while allowing ambient constructors in declared shell construction scopes.

#### Scenario: Injected core capability passes [r[injected-capability-shell-construction-acceptance.scenario.injected-core-passes]]

- GIVEN a Tigerstyle architecture fixture whose core scope accepts a named capability trait or effect data instead of reading ambient state directly
- WHEN the artifact-backed architecture check runs
- THEN no `tigerstyle.architecture.effect-capability-boundaries` diagnostic is emitted for the core fixture

#### Scenario: Shell constructor scope passes [r[injected-capability-shell-construction-acceptance.scenario.shell-scope-passes]]

- GIVEN a Tigerstyle architecture fixture whose declared shell scope owns the configured ambient constructor
- WHEN the artifact-backed architecture check runs
- THEN no `tigerstyle.architecture.effect-capability-boundaries` diagnostic is emitted for that shell fixture

### Requirement: Versioned Proof Bundle Artifacts [r[versioned-proof-bundle-artifacts]]

The system MUST export a versioned generated proof-bundle artifact for verified proof rails so downstream consumers can validate proof-contract shape before trusting proof receipts.

The proof-bundle artifact MUST include `schema_version`, `generated_from`, `registry_schema_version`, and deterministic entries that preserve policy ID, backend, command, source paths, proof paths, pass signal, trust boundaries, receipt binding, receipt provenance, and receipt verifier obligations.

#### Scenario: Proof bundle export writes versioned artifact [r[versioned-proof-bundle-artifacts.scenario.export-writes-versioned-artifact]]

- GIVEN a policy registry with at least one verified proof rail
- WHEN generated policy artifacts are exported
- THEN the output directory contains `octet-proof-bundles.json`
- AND that artifact declares `schema_version = "octet-proof-bundle/v1"`
- AND the verified rail entry preserves receipt binding, provenance, verifier, toolchain, and closure graph obligations for its policy ID

### Requirement: Proof Bundle Schema Validation [r[proof-bundle-schema-validation]]

The system MUST validate the generated proof-bundle schema when artifact-backed standards checks consume a policy artifact directory.

Artifact-backed checks MUST reject missing proof-bundle artifacts, unsupported proof-bundle schema versions, and proof-bundle entries that do not cover the verified proof rails declared by the policy artifact.

#### Scenario: Artifact-backed check rejects unsupported proof bundle schema [r[proof-bundle-schema-validation.scenario.rejects-unsupported-schema]]

- GIVEN a generated policy artifact directory whose `octet-proof-bundles.json` declares an unsupported schema version
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs against a verification fixture
- THEN the command exits non-zero
- AND the diagnostic names the unsupported proof-bundle schema version and the expected `octet-proof-bundle/v1` version

#### Scenario: Artifact-backed check accepts complete proof bundle [r[proof-bundle-schema-validation.scenario.accepts-complete-bundle]]

- GIVEN a generated policy artifact directory with a complete `octet-proof-bundles.json`
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs against a compliant verification fixture
- THEN the command exits successfully
- AND no proof-bundle schema diagnostic is emitted

### Requirement: Generated Artifact Integrity Manifest [r[generated-artifact-integrity-manifest]]

The system MUST export a generated artifact integrity manifest for policy artifact directories so downstream consumers can validate the complete generated bundle before consuming policy metadata.

#### Scenario: Artifact export writes integrity manifest [r[generated-artifact-integrity-manifest.scenario.export-writes-manifest]]

- GIVEN a policy registry exported through `tigerstyle-standards export-artifacts`
- WHEN the generated artifact directory is written
- THEN the output directory contains `octet-artifact-manifest.json`
- AND the manifest declares schema version `octet-artifact-manifest/v1`
- AND the manifest records the BLAKE3 digest and byte length for every generated artifact file except itself

### Requirement: Artifact-Backed Integrity Validation [r[artifact-backed-integrity-validation]]

The system MUST validate the generated artifact integrity manifest before artifact-backed standards checks consume a policy artifact directory.

#### Scenario: Artifact-backed check rejects tampered generated artifact [r[artifact-backed-integrity-validation.scenario.rejects-tampered-artifact]]

- GIVEN a generated policy artifact directory with a valid manifest
- AND one generated artifact file is edited after export
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs
- THEN the command exits non-zero
- AND the diagnostic names the artifact integrity manifest mismatch

#### Scenario: Artifact-backed check accepts intact generated bundle [r[artifact-backed-integrity-validation.scenario.accepts-intact-bundle]]

- GIVEN a generated policy artifact directory with an intact manifest and generated artifact files
- WHEN `tigerstyle-standards check --policy-artifacts-dir` runs
- THEN the manifest validation succeeds
- AND no artifact integrity diagnostic is emitted

### Requirement: Config and Receipt Source-of-Truth Registry [r[config-receipt-source-of-truth-registry]]

The system MUST maintain a typed registry for Tiger Style configuration and receipt families before those families add Nickel contracts, generated artifacts, or runtime validation logic.

Each family entry MUST declare a stable family ID, classification (`nickel-authored`, `rust-derived`, or `non-candidate`), owner source path, generated artifact path when applicable, generation command, validation command, status, and exclusion rationale for non-candidates.

#### Scenario: New family declares one source of truth [r[config-receipt-source-of-truth-registry.scenario.declares-one-source]]

- GIVEN a contributor adds a new run-config, policy-config, proof-receipt, deploy-receipt, or evidence-receipt surface
- WHEN the registry validation command runs
- THEN the family has exactly one source-of-truth classification
- AND the entry names the owner source path and concrete validation command
- AND registry validation rejects duplicate family IDs or families with both Nickel-authored and Rust-derived ownership

#### Scenario: Non-candidates are explicit [r[config-receipt-source-of-truth-registry.scenario.non-candidates-explicit]]

- GIVEN a proposed surface contains raw secrets, cryptographic internals, serialized capabilities, wire discriminants, distributed runtime behavior, or hot-path constants
- WHEN the registry is reviewed
- THEN the family is classified as `non-candidate`
- AND the exclusion rationale states why Nickel contracts must not own that surface

### Requirement: Nickel-Authored Human Configuration [r[nickel-authored-human-configuration]]

The system MUST use Nickel-authored contracts for human-maintained Tiger Style configuration that benefits from defaults, docs, merge overlays, environment profiles, fixture matrices, or policy knobs.

Nickel-authored config MUST export deterministic machine formats for tools while keeping helper inputs, derived locals, and non-exportable metadata behind `not_exported` fields.

#### Scenario: Human config exports deterministic tool input [r[nickel-authored-human-configuration.scenario.exports-deterministic-tool-input]]

- GIVEN a Nickel-authored config with defaults, environment overlays, and helper fields
- WHEN the config export command runs
- THEN the exported machine input is deterministic
- AND the exported data omits `not_exported` helper fields
- AND the exported data preserves policy IDs, profile names, enabled checks, timeout/retry policy, and artifact-retention settings

#### Scenario: Invalid human config fails before runtime side effects [r[nickel-authored-human-configuration.scenario.invalid-config-fails-before-effects]]

- GIVEN a Nickel-authored config with an invalid timeout ordering, unknown profile name, invalid checker reference, or impossible retention policy
- WHEN validation runs before standards checks or proof rails execute
- THEN the command exits non-zero before runtime side effects
- AND the diagnostic identifies the invalid field and config family ID

### Requirement: Rust-Derived Receipt Contracts [r[rust-derived-receipt-contracts]]

The system MUST keep runtime-emitted proof, evidence, deploy, and check receipts as Rust-owned serialized DTOs and generate Nickel contracts from those Rust-owned shapes.

Generated receipt contracts MUST preserve Rust serde names, enum alternatives, optionality, schema versions, defaults when known, and closed-record behavior only when Rust declares unknown fields invalid.

#### Scenario: Receipt contract freshness detects drift [r[rust-derived-receipt-contracts.scenario.freshness-detects-drift]]

- GIVEN a Rust receipt DTO changes its serde field names, schema version, enum alternatives, or optionality
- WHEN the generated-contract freshness check runs without updating the Nickel artifact
- THEN the check fails with a diff or diagnostic naming the stale generated contract
- AND the registry still identifies the Rust source path as authoritative

#### Scenario: Receipt fixtures validate shape without owning behavior [r[rust-derived-receipt-contracts.scenario.fixtures-validate-shape-not-behavior]]

- GIVEN positive and negative receipt JSON fixtures for a Rust-derived receipt family
- WHEN Nickel contract validation runs against those fixtures
- THEN valid fixtures satisfy the generated contract
- AND invalid fixtures fail only for Rust-owned shape, schema-version, required-field, or explicitly modeled invariant violations
- AND Nickel validation does not claim to prove distributed behavior, cryptographic correctness, or runtime transition semantics

### Requirement: Structured Acceptance Receipts [r[structured-acceptance-receipts]]

The system MUST treat structured receipts, not raw logs, as acceptance records for proof/evidence/config review.

Acceptance receipts MUST include schema version, run or receipt ID, timestamps, tool/version or git revision, config digest for Nickel-authored inputs, inputs, stage outcomes, artifacts, status, warnings/errors, and verification commands or evidence references when applicable.

#### Scenario: Artifact export emits digest-bound receipt [r[structured-acceptance-receipts.scenario.artifact-export-emits-digest-bound-receipt]]

- GIVEN an exported Nickel run profile JSON and a policy registry JSON
- WHEN `octet-standards export-artifacts --profile-json <profile> --receipt-out <receipt>` runs
- THEN the command writes a structured acceptance receipt
- AND the receipt `config_digest` equals the BLAKE3 digest of the profile JSON bytes
- AND the receipt lists the generated artifact bundle and verification command evidence without embedding raw logs

### Requirement: Octet Effect Capability Vocabulary [r[octet-effect-capability-vocabulary]]

The system MUST maintain a stable Octet capability vocabulary for effect-like authority that includes time, randomness, filesystem, network, process spawning, environment variables, secrets, remote execution, unsafe code, global state, and concurrency.

#### Scenario: Policy registry names capability family [r[octet-effect-capability-vocabulary.scenario.registry-names-family]]

- GIVEN a policy or lint detects an ambient effect or shell authority
- WHEN the policy registry is exported
- THEN the entry includes a stable Octet capability family name
- AND the family name belongs to the approved capability vocabulary

#### Scenario: FCIS guidance classifies core and shell authority [r[octet-effect-capability-vocabulary.scenario.fcis-classifies-authority]]

- GIVEN a policy describes deterministic functional core / imperative shell boundaries
- WHEN a reader reviews the capability vocabulary
- THEN ambient time, randomness, filesystem, network, process, environment, secrets, remote execution, unsafe, global state, and concurrency authority are classified as core-prohibited unless represented as explicit data or named capability traits
- AND shell-allowed uses are documented as interpreter responsibilities

### Requirement: Octet Effect Capability Mapping Validation [r[octet-effect-capability-mapping-validation]]

The system MUST validate that effect-like lint and policy outputs either map to a known capability family or explicitly report an unknown capability status. Unknown status MUST be visible in human and machine-readable summaries.

#### Scenario: Known capability appears in output [r[octet-effect-capability-mapping-validation.scenario.known-capability-output]]

- GIVEN a lint finding is mapped to the `time` or `filesystem` capability family
- WHEN JSON, SARIF, or human summary output is generated
- THEN the output includes the mapped capability family
- AND effect summaries count the finding under that family

#### Scenario: Unknown capability is visible [r[octet-effect-capability-mapping-validation.scenario.unknown-capability-visible]]

- GIVEN a new lint has no capability-family mapping
- WHEN policy validation or summary output runs
- THEN the result reports an explicit unknown capability status
- AND the unknown status is not omitted from effect summaries

### Requirement: Octet Review Identity Registry [r[octet-review-identity-registry]]

The system MUST support a review identity registry that maps mutable human-readable review labels, evidence aliases, policy names, and fingerprint manifest handles to immutable content-addressed targets.

#### Scenario: Label resolves to immutable review target [r[octet-review-identity-registry.scenario.label-resolves-target]]

- GIVEN a fingerprint review manifest references a review label or evidence alias
- WHEN registry validation runs
- THEN the label resolves to an immutable target hash and target kind
- AND the manifest output preserves both the human label and immutable target hash

#### Scenario: Pointer history supports rollback [r[octet-review-identity-registry.scenario.pointer-rollback]]

- GIVEN a review label is updated to point at a newer evidence target
- WHEN pointer history is inspected or rolled back
- THEN the previous immutable target remains visible
- AND rollback restores the label to the prior target without mutating either target hash

### Requirement: Octet Review Identity Diagnostics [r[octet-review-identity-diagnostics]]

The system MUST diagnose stale, missing, or mismatched review identity pointers without claiming that identity matches prove semantic correctness.

#### Scenario: Missing identity is diagnosed [r[octet-review-identity-diagnostics.scenario.missing-identity]]

- GIVEN a fingerprint review manifest names a registry label that does not exist
- WHEN strict review identity validation runs
- THEN validation fails with the missing label and expected target kind
- AND the diagnostic names the manifest entry that referenced it

#### Scenario: Identity match avoids correctness claim [r[octet-review-identity-diagnostics.scenario.no-correctness-claim]]

- GIVEN a review identity pointer resolves and the fingerprint hash matches
- WHEN human or machine output is generated
- THEN the output states that the reviewed identity and source body match
- AND it does not state that the function is correct, safe, or verified solely because the identity matches

### Requirement: Octet-owned function object v1

r[content_addressed_rust_evidence.function_object_v1] Octet MUST emit deterministic function-object v1 records for selected Rust functions as source-token review/provenance identities with explicit schema, normalization, toolchain, body hash, signature hash, object hash, source span, semantic id, and caveats.

#### Scenario: Emit source-token function object

- GIVEN a selected Rust function accepted by Octet's fingerprint scanner
- WHEN the user requests a function-object v1 record
- THEN Octet emits canonical JSON containing the schema id, semantic id, source span, body hash, signature hash, normalization version, toolchain identity, object hash, and caveats declaring source-token identity.

#### Scenario: Reject correctness overclaim

- GIVEN a function-object v1 record whose hashes match current source
- WHEN Octet renders JSON, human output, or review text for that object
- THEN the output describes the object as review/provenance evidence and not as a proof of behavioral correctness or semantic equivalence.

#### Scenario: Real-code corpus emits stable source objects

- GIVEN a bounded corpus of Octet's real Rust implementation modules
- WHEN Octet emits function-object v1 records across that corpus twice
- THEN the emitted object set is deterministic, covers many real functions, preserves per-file source paths, reports caveat distribution, and keeps correctness claims false.

### Requirement: Canonical object hashing

r[content_addressed_rust_evidence.canonical_hash] Octet MUST compute object hashes with BLAKE3 over deterministic canonical object content that excludes self-referential hash fields and ambient entropy.

#### Scenario: Same logical object hashes the same across roots

- GIVEN the same logical selected function emitted from two different repository root paths
- WHEN Octet computes the function-object v1 hash
- THEN display-only path differences do not change the hash input and the object hashes match.

#### Scenario: Entropy-bearing fields are excluded or rejected

- GIVEN object fields derived from timestamps, temporary absolute paths, environment variables, or process-local state
- WHEN Octet prepares canonical hash input
- THEN those fields are either excluded as display metadata or rejected as non-canonical hash material.

### Requirement: Object marker compatibility

r[content_addressed_rust_evidence.object_markers] Octet MUST support inline object markers that bind selected Rust functions to function-object hashes while preserving existing fingerprint marker workflows.

#### Scenario: Object marker check detects drift

- GIVEN a function with an inline object marker referencing a prior object hash
- WHEN the function body, signature, or caveat-relevant context changes
- THEN Octet check reports drift and requires an explicit update before the object is considered reviewed.

#### Scenario: Existing fingerprint workflows remain usable

- GIVEN a repository that only uses existing Octet function fingerprint markers
- WHEN object support is added
- THEN existing fingerprint check, write, update, and diff workflows continue to work without requiring object markers.

### Requirement: Name pointer registry

r[content_addressed_rust_evidence.name_pointers] Octet MUST support a deterministic name-pointer manifest that maps mutable Rust semantic names to immutable object hashes and records pointer movement metadata.

#### Scenario: Pointer move records old and new identity

- GIVEN a name pointer for `crate::module::function` that references an old object hash
- WHEN the user moves that pointer to a new object hash
- THEN Octet records the old hash, new hash, semantic name, diff summary, and rollback handle in deterministic output.

#### Scenario: Pointer references must resolve

- GIVEN a name-pointer manifest and an object bundle
- WHEN Octet validates the manifest
- THEN every pointer hash resolves to a present object record or validation fails closed.

### Requirement: Structured dependency and effect caveats

r[content_addressed_rust_evidence.caveats] Octet MUST emit machine-readable dependency, macro, generated-code, and effect caveats using closed status values that distinguish source-token scanning from future HIR/MIR evidence.

#### Scenario: Source-scanned dependencies are not reported as resolved

- GIVEN a function-object v1 record produced by the source scanner
- WHEN Octet emits dependency metadata
- THEN the dependency status uses a source-scanner status and does not claim rustc/HIR-resolved dependency identity.

#### Scenario: Unknown effects block pure-cache eligibility

- GIVEN a function object with unknown, macro-hidden, generated, or unresolved effect sources
- WHEN Octet emits a pure-cache eligibility summary
- THEN the object is reported as ineligible unless stronger reviewed evidence is present.

### Requirement: Object diff

r[content_addressed_rust_evidence.object_diff] Octet MUST provide object-aware diffs that report body, signature, dependency caveat, effect caveat, macro caveat, and name-pointer changes separately.

#### Scenario: Diff separates body and caveat changes

- GIVEN old and new function-object records for the same semantic name
- WHEN Octet compares them
- THEN the diff reports whether body hash, signature hash, dependency caveats, effect caveats, macro caveats, or pointer targets changed as separate fields.

### Requirement: Export bundle

r[content_addressed_rust_evidence.export_bundle] Octet MUST export deterministic evidence bundles for downstream tools containing object records, name pointers, review manifests, caveat summaries, and replay metadata.

#### Scenario: Bundle membership is deterministic

- GIVEN a set of selected function objects and name pointers
- WHEN Octet exports a bundle
- THEN bundle membership and manifest ordering are deterministic and the bundle hash is stable for the same logical content.

#### Scenario: Bundle records replay command

- GIVEN an exported bundle
- WHEN a downstream consumer reads its receipt metadata
- THEN the metadata records the Octet command, relevant config/normalization versions, and input selection needed to reproduce the bundle.

#### Scenario: Real-code corpus bundle is deterministic

- GIVEN function objects emitted from a bounded corpus of Octet's real Rust implementation modules
- WHEN Octet exports the corpus bundle repeatedly
- THEN the bundle bytes, object count, bundle hash, and replay command are stable for the same corpus selection.

### Requirement: Real-code harness hardening

r[content_addressed_rust_evidence.real_code_harness] Octet MUST provide a repeatable real-code harness for function-object emission, caveat summaries, negative case classification, and bundle determinism before adding compiler IR-derived evidence.

#### Scenario: Repeatable real-code harness emits classified summary

- GIVEN a named corpus manifest of Octet implementation source files
- WHEN the harness emits function-object v1 records repeatedly
- THEN it compares stable object output, reports object count, source-path coverage, macro/effect caveat distribution, pure-cache blocking, and no-correctness-overclaim status.

#### Scenario: Negative corpus cases are classified

- GIVEN fixture mutations for body changes, comment/formatting-only changes, macro-heavy functions, missing pointers, and unresolved bundle pointers
- WHEN the harness runs those cases
- THEN it proves expected hash stability or drift and fails closed with classified caveat or missing-object outcomes.

#### Scenario: Harness bundle export is deterministic

- GIVEN objects emitted from the named real-code corpus
- WHEN the harness exports bundles repeatedly
- THEN bundle bytes, object count, pointer count, bundle hash, and replay command are stable, and unresolved pointer manifests fail closed.

#### Scenario: Strong local determinism receipt

- GIVEN a scrubbed local environment, multiple fresh target directories, a copied corpus root, and permuted corpus input order
- WHEN the harness emits objects and bundles ten times
- THEN the receipt records binary, toolchain, system, corpus file hashes, stable emit byte hash, stable bundle byte hash, cross-root object-set equivalence, and input-order object-set equivalence.

### Requirement: Compiler IR evidence roadmap

r[content_addressed_rust_evidence.compiler_ir] Octet MUST treat future HIR, THIR, and MIR evidence as supplemental compiler-version-coupled evidence layers that improve dependency/effect summaries without replacing function-object v1 review identity.

#### Scenario: HIR evidence is toolchain scoped

- GIVEN a future object containing HIR-derived dependency evidence
- WHEN Octet emits that evidence
- THEN the record includes rustc/toolchain identity, extractor version, resolved dependency summary, and unresolved or macro-origin caveats.

#### Scenario: MIR evidence does not claim cross-version stability

- GIVEN a future object containing MIR-derived call, panic, drop, async, or unsafe summaries
- WHEN Octet emits that evidence
- THEN the record identifies the exact rustc/toolchain and avoids claiming stability across compiler versions.

### Requirement: Downstream evidence boundaries

r[content_addressed_rust_evidence.downstream_boundaries] Octet MUST define downstream evidence boundaries for Valence, Cairn, Trellis, Crunch/Mantle, and Aspen that preserve Octet as the Rust extraction/object source while avoiding claims owned by those systems.

#### Scenario: Cairn consumes Octet receipts without owning extraction

- GIVEN an Octet export bundle receipt
- WHEN Cairn consumes it
- THEN Cairn validates lifecycle/replay policy around the receipt while Octet remains the authority for Rust extraction and object emission.

#### Scenario: Crunch and Aspen pilots remain bounded

- GIVEN Crunch/Mantle or Aspen consumes Octet evidence objects
- WHEN Octet describes the interop contract
- THEN the contract identifies checker/config/source/executor hashes as evidence identity and does not claim build correctness or distributed execution correctness from Octet hashes alone.

### Requirement: Function Object Internal Corpus Pilot [r[function-object-internal-corpus-pilot]]

The system MUST support an Octet-internal function-object corpus pilot that emits deterministic machine-readable receipt summaries for a bounded set of Octet Rust source files.

The receipt MUST include the selected source paths, object count, deterministic object-set hash, caveat distribution, pure-cache blocking count, replay command, and an explicit boundary statement that the receipt is review/provenance evidence rather than a correctness or semantic-equivalence proof.

#### Scenario: Expanded internal corpus emits a deterministic receipt

- GIVEN a bounded Octet source corpus broader than the original object/fingerprint/CLI three-file pilot
- WHEN the corpus receipt command runs twice against the same inputs
- THEN both JSON receipts are byte-identical
- AND the receipt records all selected source paths and an object count greater than the original three-file pilot threshold

#### Scenario: Receipt preserves no-correctness boundary

- GIVEN the internal object corpus contains functions with unresolved dependency, effect, or macro caveats
- WHEN the corpus receipt is emitted
- THEN the receipt reports caveat counts and pure-cache blockers
- AND it states that object hashes are review/provenance evidence, not correctness proofs or semantic equivalence proofs

### Requirement: Function Object Corpus Receipt Check [r[function-object-corpus-receipt-check]]

The system MUST support replaying a saved function-object corpus receipt against selected Rust source inputs and reporting whether the current evidence identity still matches the saved receipt.

The check MUST compare receipt-critical evidence fields, including schema, object count, selected source paths, object-set hash, caveat counts, pure-cache blocker count, and no-correctness/no-semantic-equivalence boundary fields. The check MUST fail closed on drift, malformed receipts, unsupported schema, unsupported schema versions, missing required fields, invalid source selections, empty corpora, or boundary tampering.

Invalid receipt or selection failures MUST emit a deterministic `object corpus check` report with `status = "invalid"` and typed errors when JSON or human output is requested. These failure reports MUST preserve the boundary that receipt checks validate review/provenance evidence identity only; they MUST NOT claim behavioral correctness, semantic equivalence, or cross-machine determinism.

Object corpus receipt checks MUST preserve named compatibility fixture coverage for accepted v1 receipts and fail-closed malformed/tampered receipt or source-selection examples without changing the v1 receipt schema.

#### Scenario: Saved receipt replays cleanly

- GIVEN a saved function-object corpus receipt for selected Rust source files
- WHEN the corpus check command is run with the same source inputs
- THEN the command exits successfully
- AND the report marks schema, object count, source paths, object-set hash, caveat counts, pure-cache blocker count, and boundary fields as matching

#### Scenario: Source drift fails the receipt check

- GIVEN a saved function-object corpus receipt
- WHEN a selected source function changes before the check is rerun
- THEN the command exits non-zero
- AND the report identifies at least the object-set hash drift without claiming semantic equivalence

#### Scenario: Boundary tampering fails the receipt check

- GIVEN a saved corpus receipt whose no-correctness or no-semantic-equivalence boundary was altered
- WHEN the corpus check command is run
- THEN the command exits non-zero
- AND the report identifies boundary drift

#### Scenario: Malformed receipt emits typed invalid report

- GIVEN a saved receipt file that is not valid JSON
- WHEN the corpus check command is run with JSON output
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with an `invalid-json` error kind

#### Scenario: Unsupported schema is rejected before comparison

- GIVEN a saved receipt with an unknown schema or future schema version
- WHEN the corpus check command is run
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with a typed schema error

#### Scenario: Missing receipt fields are rejected before comparison

- GIVEN a saved receipt missing a required critical field
- WHEN the corpus check command is run
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with a `missing-field` error naming the field

#### Scenario: Invalid source selection is rejected deterministically

- GIVEN a check invocation with missing, duplicate, non-Rust, or empty-corpus source selection
- WHEN the corpus check command is run with JSON output
- THEN the command exits non-zero
- AND the report has `status = "invalid"` with typed source-selection errors

#### Scenario: Boundary tamper fixture fails closed
- GIVEN a checked-in v1 receipt fixture whose `boundary.claims_correctness` value is tampered to `true`
- WHEN `cargo octet object corpus check` replays it against the compatibility source fixture
- THEN the check MUST fail closed with a `mismatch` report naming the `boundary` comparison field.

#### Scenario: Duplicate receipt source path fixture fails closed
- GIVEN a checked-in v1 receipt fixture whose `source_paths` array repeats the same source file
- WHEN `cargo octet object corpus check` validates it
- THEN the check MUST fail closed with an `invalid` report containing `duplicate-source-path` for `source_paths`.

#### Scenario: Malformed receipt fixture fails closed
- GIVEN a checked-in malformed receipt JSON fixture
- WHEN `cargo octet object corpus check` validates it
- THEN the check MUST fail closed with an `invalid` report containing `invalid-json` for `receipt`.

#### Scenario: Empty receipt source path fixture fails closed
- GIVEN a checked-in v1 receipt fixture whose `source_paths` array is empty
- WHEN `cargo octet object corpus check` validates it against the compatibility source fixture
- THEN the check MUST fail closed with an `invalid` report containing `empty-source-selection` for `source_paths`.

#### Scenario: Unsupported non-Rust source selection fixture fails closed
- GIVEN the checked-in positive v1 compatibility receipt and a checked-in non-Rust source-selection fixture
- WHEN `cargo octet object corpus check` is invoked with that non-Rust source path
- THEN the check MUST fail closed with an `invalid` report containing `unsupported-source-path` for `paths`.

### Requirement: Internal Object Corpus Gate

The system MUST provide a checked-in gate fixture and deterministic replay test for Octet's bounded internal function-object corpus.

The gate MUST compare the saved receipt against current selected source inputs using `cargo octet object corpus check --output-format json`, and it MUST preserve the boundary that matching object corpus receipts validate review/provenance evidence identity only.

#### Scenario: Checked-in internal corpus receipt replays cleanly

- GIVEN a checked-in internal object corpus receipt fixture for the bounded Octet source corpus
- WHEN the gate replays the fixture against current selected source paths
- THEN the gate exits successfully and every receipt-critical comparison reports `match`

#### Scenario: Internal corpus gate preserves evidence-only boundaries

- GIVEN the checked-in corpus receipt and replay report
- WHEN the gate inspects the boundary comparison
- THEN the report preserves `claims_correctness = false` and `claims_semantic_equivalence = false` without claiming behavioral correctness or semantic equivalence

#### Scenario: Fixture refresh is deterministic

- GIVEN the checked-in internal corpus receipt fixture
- WHEN a fresh receipt is generated from the same bounded source corpus
- THEN the fresh receipt bytes match the checked-in fixture or the review must include an intentional fixture update

### Requirement: Octet Evidence File Format Contract

Octet MUST distinguish human review artifacts from machine receipt artifacts for content-addressed Rust evidence and define the supported format role for each artifact class.

#### Scenario: Markdown remains human review surface

- GIVEN an Octet OpenSpec change, design note, task list, archived evidence note, or review narrative for content-addressed Rust evidence
- WHEN the artifact is intended for human review and lifecycle context
- THEN Markdown MUST remain an accepted review format
- AND the artifact MUST state caveats in prose when it discusses object identity, determinism, replay, or boundary claims.

#### Scenario: JSON remains current machine receipt surface

- GIVEN an Octet function object, object bundle, corpus receipt, corpus check report, deterministic invalid-input report, or checked-in receipt fixture
- WHEN the artifact is emitted for machine replay, CI gating, fixture comparison, or bundle inclusion
- THEN deterministic JSON MUST remain an accepted v1 receipt encoding
- AND the receipt MUST carry explicit schema/version identity and deterministic hash boundaries.

### Requirement: Octet Receipt Contract Ownership

Octet MUST treat emitted evidence receipt compatibility as Rust DTO-owned runtime behavior with explicit schema/version identifiers and, where policy-relevant, generated or registered Nickel contract metadata.

#### Scenario: Rust DTO owns emitted receipt shape

- GIVEN an emitted Octet evidence receipt family consumed by Octet tests, CI, Cairn, or downstream reviewers
- WHEN the receipt shape is validated for compatibility
- THEN the Rust DTO and serializer behavior MUST be the runtime source of truth
- AND compatibility changes MUST be versioned rather than inferred only from `.json` file names.

#### Scenario: Nickel metadata tracks compatibility policy

- GIVEN an Octet evidence receipt family that becomes compatibility-relevant across tools or policies
- WHEN repository policy describes the family
- THEN the receipt family MUST be registered as Rust-derived, Nickel-authored, or non-candidate in Nickel contract/catalog metadata
- AND hand-authored JSON schema MUST NOT become the normative compatibility source when it can drift from the Rust DTO.

### Requirement: Canonical Serialization Format Boundaries

Octet MUST preserve deterministic canonical serialization rules for current JSON receipts while allowing future additive encodings only when they retain schema identity, hash boundaries, and reviewable summaries.

#### Scenario: JSON hash material is canonical and bounded

- GIVEN a deterministic JSON Octet receipt with object, corpus, bundle, or check hash fields
- WHEN hash material is computed or validated
- THEN self-referential hash fields and ambient host entropy MUST be excluded from hash material
- AND sorted deterministic content MUST be used for portable evidence hashes.

#### Scenario: Future binary format is additive

- GIVEN a future canonical binary or packed evidence bundle format for Octet receipts
- WHEN the format is introduced
- THEN it MUST be additive to the v1 JSON review surface unless a later migration change explicitly replaces that surface
- AND it MUST provide schema identity, compatibility rules, deterministic hashes, and human-reviewable summaries.

### Requirement: File Format Non-Claims

Octet MUST require content-addressed evidence file-format receipts to preserve explicit evidence-only boundaries and reject correctness or semantic-equivalence claims derived only from artifact format or hash agreement.

#### Scenario: File format does not prove correctness

- GIVEN a Markdown narrative, JSON receipt, or future binary bundle whose hashes validate
- WHEN Octet or a downstream consumer reports the artifact status
- THEN the report MUST state or preserve that hash agreement is evidence/provenance identity, not a behavioral correctness proof.

#### Scenario: File format does not prove semantic equivalence

- GIVEN two receipts or bundles that share matching object/corpus hash fields
- WHEN the artifacts are compared
- THEN the comparison MUST NOT claim Rust semantic equivalence solely from matching file-format encodings or hashes.

### Requirement: Function Object Corpus Contract Metadata [r[function-object-corpus-contract-metadata]]

Octet MUST expose Rust-derived contract metadata for `octet.function-object-corpus-receipt.v1` and its corpus check report so review and CI can verify schema names, schema versions, required fields, compared fields, and boundary non-claims without treating JSON files as the source of truth.

#### Scenario: Receipt contract metadata

- GIVEN the object corpus receipt DTO and schema/version constants in Rust
- WHEN an operator runs the object corpus contract metadata command
- THEN the JSON output names `octet.function-object-corpus-receipt.v1`, schema version `1`, required receipt fields, compared replay fields, and the no-correctness/no-semantic-equivalence boundary fields

#### Scenario: Check-report contract metadata

- GIVEN the object corpus check report DTO in Rust
- WHEN an operator runs the object corpus contract metadata command
- THEN the JSON output names the check report schema/version, required report fields, valid status values, and required error/comparison fields

#### Scenario: Checked-in fixture compatibility

- GIVEN the checked-in internal object corpus receipt fixture
- WHEN the fixture is validated by contract metadata tests
- THEN required fields are present, schema/version match the Rust constants, and `claims_correctness` plus `claims_semantic_equivalence` remain false

#### Scenario: Missing required field fails closed

- GIVEN a corpus receipt with a field required by contract metadata removed
- WHEN the corpus check path validates it
- THEN the result is a deterministic invalid-input report naming the missing field

### Requirement: Function Object Corpus Fresh-Target Determinism [r[function-object-corpus-fresh-target-determinism]]

Octet MUST verify that object corpus receipt generation remains stable across repeated local runs that use distinct fresh target directories and a scrubbed deterministic environment, while preserving the receipt boundary that hashes identify review/provenance evidence rather than correctness or semantic equivalence.

#### Scenario: Fresh target byte stability

- GIVEN the internal object corpus source selection
- WHEN Octet generates object corpus receipts multiple times with distinct fresh `CARGO_TARGET_DIR` values
- THEN the generated receipt bytes match exactly across runs

#### Scenario: Fresh target compared field stability

- GIVEN the generated fresh-target receipts
- WHEN the test compares the replay-significant receipt fields
- THEN object count, source paths, object-set hash, caveat counts, pure-cache blocker count, and boundary fields match across runs

#### Scenario: Fresh target bounded non-claims

- GIVEN the generated fresh-target receipts
- WHEN the boundary fields are inspected
- THEN `claims_correctness` and `claims_semantic_equivalence` remain false and the evidence is described as bounded local repeatability rather than cross-machine or semantic proof

### Requirement: Function Object Corpus CI Gate [r[function-object-corpus-ci-gate]]

Octet MUST expose a named CI/check target for the internal function object corpus evidence gate so fixture replay, fixture freshness, Rust-derived contract metadata compatibility, and fresh-target receipt determinism can be run without rediscovering individual test names.

#### Scenario: Object corpus CI check covers fixture replay

- GIVEN the checked-in internal object corpus receipt fixture
- WHEN the named object corpus CI/check target runs
- THEN it MUST execute the fixture replay and byte-for-byte fixture freshness assertions for the selected internal corpus

#### Scenario: Object corpus CI check covers contract metadata

- GIVEN the Rust-derived object corpus receipt/check-report contract metadata
- WHEN the named object corpus CI/check target runs
- THEN it MUST execute the fixture compatibility assertions for required fields, schema/version identity, and boundary fields

#### Scenario: Object corpus CI check covers fresh-target determinism

- GIVEN the fresh-target object corpus determinism rail
- WHEN the named object corpus CI/check target runs
- THEN it MUST execute receipt generation across fresh target directories and assert stable replay-significant fields

#### Scenario: Object corpus CI check preserves bounded non-claims

- GIVEN a passing named object corpus CI/check target
- WHEN the result is documented or used as evidence
- THEN it MUST be described as bounded local evidence replay for the selected corpus and MUST NOT claim correctness, semantic equivalence, cross-machine determinism, or exhaustive Rust coverage

### Requirement: Object corpus compatibility fixtures

Octet MUST maintain checked-in compatibility fixtures for object corpus receipt schema v1 that document accepted replay behavior and fail-closed rejection behavior.

#### Scenario: Positive v1 fixture replays cleanly

- GIVEN a checked-in v1 object corpus receipt compatibility fixture and its checked-in Rust source corpus
- WHEN `cargo octet object corpus check` replays that fixture against the source corpus
- THEN the check report MUST return `status=match` with matching compared fields
- AND the fixture MUST keep `claims_correctness=false` and `claims_semantic_equivalence=false`

#### Scenario: Negative fixtures reject unsupported compatibility shapes

- GIVEN checked-in negative object corpus receipt compatibility fixtures for a missing required field and a future schema version
- WHEN `cargo octet object corpus check` evaluates those fixtures against the source corpus
- THEN each check report MUST return `status=invalid`
- AND each report MUST include the expected typed error kind
- AND comparisons MUST remain empty for invalid fixtures

### Requirement: Expanded internal object corpus taxonomy

Octet MUST maintain an expanded internal object corpus taxonomy that selects stable in-repository Rust source modules beyond `cargo-octet/src/*.rs`, records the selected paths in a checked-in fixture, and validates a deterministic object corpus receipt for that expanded source set while preserving bounded evidence claims.

#### Scenario: Expanded taxonomy names selected source buckets

- GIVEN the expanded internal object corpus taxonomy fixture
- WHEN the taxonomy is inspected by tests
- THEN it names the selected source paths from both the `cargo-octet` and `octet-standards` buckets
- AND every selected path is present in the expanded corpus receipt fixture.

#### Scenario: Expanded corpus fixture replays cleanly

- GIVEN the expanded internal object corpus receipt fixture
- WHEN Octet checks it against the taxonomy-selected source paths
- THEN the check report status is `match`
- AND every compared field matches.

#### Scenario: Expanded corpus preserves bounded evidence claims

- GIVEN the expanded internal object corpus receipt fixture
- WHEN tests inspect its boundary fields
- THEN `claims_correctness` is false
- AND `claims_semantic_equivalence` is false.

### Requirement: Fresh-build object corpus determinism rail

Octet MUST provide a named fresh-build object corpus determinism rail that regenerates the expanded internal object corpus receipt with independently built Octet binaries and compares deterministic receipt identity while preserving bounded evidence claims.

#### Scenario: Fresh-build binaries reproduce expanded corpus receipt

- GIVEN the checked-in expanded internal object corpus taxonomy and receipt fixture
- WHEN the fresh-build object corpus rail runs independent Nix-built `cargo-octet` binaries over the selected corpus
- THEN the generated receipts match each other and the checked-in expanded receipt fixture for bytes and stable contract fields

#### Scenario: Fresh-build rail preserves bounded non-proof claims

- GIVEN a fresh-build generated object corpus receipt
- WHEN the rail inspects the receipt boundary fields
- THEN `claims_correctness` is false, `claims_semantic_equivalence` is false, and the rail describes the result as bounded local fresh-build evidence replay rather than correctness, semantic equivalence, cross-machine determinism, or exhaustive Rust coverage proof

#### Scenario: Contributor guide names the fresh-build replay check

- GIVEN contributors need to replay fresh-build object corpus evidence locally
- WHEN they read the contributor guide
- THEN it names `nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism --no-link -L` and preserves bounded local non-proof language

### Requirement: HIR/MIR object evidence supplement contract

Octet MUST define HIR/MIR object evidence as optional, toolchain-scoped supplements to `function-object-v1` rather than replacements for source-token object identity. For the bounded pilot, any emitted supplement MUST be attached to a selected Function Object v1 object, MUST include exact toolchain/extractor/schema identity, and MUST include explicit caveats for every unsupported or incomplete evidence family.

#### Scenario: HIR supplement preserves v1 identity boundary

- GIVEN a Function Object v1 record with source-token object identity
- WHEN a pilot HIR evidence supplement is attached to that object
- THEN the supplement records its rustc/toolchain identity, extractor identity, schema version, and extraction caveats
- AND the Function Object v1 `object_hash` remains a review/provenance identity rather than a correctness or semantic-equivalence proof

#### Scenario: MIR supplement is version and context scoped

- GIVEN a pilot MIR evidence supplement for an object
- WHEN the supplement records MIR-derived observations
- THEN it records the rustc version, extractor identity, MIR context or optimization assumptions, and unsupported-context caveats
- AND it does not claim cross-toolchain stability, behavioral correctness, or semantic equivalence from matching MIR evidence

#### Scenario: Supplement metadata fails closed before implementation claims

- GIVEN Octet has not implemented a HIR, THIR, or MIR extractor for a requested evidence family with deterministic fixture evidence
- WHEN contract metadata or operator documentation describes HIR/MIR evidence support
- THEN it reports the family as planned or unsupported rather than silently emitting partial support
- AND any future implementation must add deterministic fixture evidence before claiming support

### Requirement: HIR/MIR supplement contract metadata

Octet MUST expose HIR/MIR supplemental evidence families in object corpus contract metadata with closed support statuses. Families without deterministic extractor fixtures MUST remain `unsupported-planned`; bounded pilot families with positive and negative fixture evidence MAY report `pilot-supported` while preserving toolchain-scoped non-proof boundaries.

#### Scenario: Contract metadata names planned HIR/MIR supplements

- GIVEN `cargo octet object corpus contract --output-format json`
- WHEN the contract metadata is inspected
- THEN it lists HIR, THIR, and MIR supplemental evidence families with schema IDs, schema versions, required future fields, support status, and relationship to Function Object v1

#### Scenario: Planned supplements fail closed before extractor implementation

- GIVEN Octet has not implemented deterministic fixtures for a HIR, THIR, or MIR supplement family
- WHEN contract metadata describes that supplement status
- THEN the family reports `unsupported-planned` rather than supported or partial support
- AND the family records an unsupported reason and required deterministic fixture evidence before support can be claimed

#### Scenario: Pilot-supported metadata remains bounded

- GIVEN a supplement family has deterministic positive and negative pilot fixtures
- WHEN contract metadata reports that family as `pilot-supported`
- THEN it names the exact schema, toolchain scope, extractor identity expectations, required receipt/check fields, and fixture replay command
- AND it states the pilot is bounded supplement evidence, not correctness, semantic-equivalence, pure-cache, cross-toolchain, or exhaustive Rust coverage proof

### Requirement: Internal corpus caveat taxonomy report

Octet MUST provide a deterministic object corpus caveat taxonomy report for selected Rust source inputs that explains per-source-bucket dependency, effect, macro, and pure-cache blockers while preserving evidence-only boundaries.

#### Scenario: Expanded internal taxonomy fixture is deterministic
- GIVEN the checked-in expanded internal object corpus source taxonomy
- WHEN Octet generates the object corpus caveat taxonomy report as JSON
- THEN the generated report bytes match the checked-in caveat taxonomy fixture.

#### Scenario: Caveat taxonomy names per-source blockers
- GIVEN selected internal corpus source paths with dependency, effect, macro, or pure-cache blockers
- WHEN Octet emits the caveat taxonomy report
- THEN each source bucket reports object count, caveat counts, pure-cache blocked count, and blocker names deterministically.

#### Scenario: Caveat taxonomy preserves non-proof boundaries
- GIVEN an object corpus caveat taxonomy report
- WHEN boundary fields are inspected
- THEN `claims_correctness` and `claims_semantic_equivalence` remain false and the report does not claim pure-cache eligibility or semantic equivalence.

### Requirement: Aggregate Valence Object Evidence Gate [r[aggregate-valence-object-evidence-gate]]

The system MUST provide a named Nix check that aggregates Octet's selected Valence object-evidence rails for operator and CI use.

The aggregate gate MUST compose the checked-in internal object corpus replay rail, object corpus compatibility fixture rail, object corpus caveat taxonomy rail, object corpus fixture freshness rail, supplement compatibility rail, bounded local fresh-build object corpus determinism rail, THIR supplement fixture inventory rail, and MIR supplement fixture inventory rail. It MUST preserve the existing non-proof boundaries: object evidence identifies reviewed evidence/provenance bytes and replay inputs, not behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-machine/cross-toolchain determinism. The THIR and MIR inventory rails MUST remain selected-fixture-only and MUST NOT imply general THIR or MIR extraction.

#### Scenario: Aggregate gate composes object evidence rails

- GIVEN a maintainer invokes the aggregate Valence object evidence Nix check
- WHEN the check evaluates and builds
- THEN it depends on the object corpus replay, compatibility, caveat taxonomy, fixture freshness, supplement compatibility, fresh-build determinism, THIR supplement fixture inventory, and MIR supplement fixture inventory rails
- AND it exits successfully only when all composed rails succeed

#### Scenario: Aggregate gate composes supplement compatibility rail [r[aggregate-valence-object-evidence-gate.scenario.supplement-compatibility-rail]]

- GIVEN the focused supplement compatibility rail checks positive and negative supplement receipt replay behavior
- WHEN the aggregate Valence object evidence check builds
- THEN it depends on that supplement compatibility rail
- AND its output reports `valence_object_evidence.supplement_compatibility_gate: PASS`
- AND the aggregate non-claims continue to exclude correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, and cross-toolchain determinism

#### Scenario: Aggregate gate composes MIR inventory rail [r[aggregate-valence-object-evidence-gate.scenario.mir-inventory-rail]]

- GIVEN the focused MIR supplement inventory rail checks the selected MIR fixture inventory and inventory-listed emit/check commands
- WHEN the aggregate Valence object evidence check builds
- THEN it depends on that MIR inventory rail
- AND its output reports `valence_object_evidence.mir_supplement_inventory_gate: PASS`
- AND the existing `valence_object_evidence.mir_supplement_fixture_gate: PASS` line remains available until any compatibility rename is explicitly retired

#### Scenario: Contributor docs name the aggregate command and MIR boundaries

- GIVEN a contributor reads the contributor guide
- WHEN they look for Valence object-evidence verification
- THEN the guide names the aggregate Nix check command
- AND it states the aggregate gate is bounded evidence replay rather than a correctness, semantic-equivalence, exhaustive-coverage, or cross-machine determinism proof
- AND it states the aggregate includes selected THIR and MIR fixture inventory coverage without implying general THIR or MIR extraction

#### Scenario: Aggregate gate keeps THIR branch-control-flow inventory visible [r[aggregate-valence-object-evidence-gate.scenario.thir-branch-control-flow-aggregate-visible]]

- GIVEN the focused THIR supplement inventory rail checks the selected THIR fixture inventory and inventory-listed emit/check commands
- WHEN the aggregate Valence object evidence check builds
- THEN it depends on that THIR inventory rail
- AND its output reports `valence_object_evidence.thir_supplement_inventory_gate: PASS`
- AND the aggregate non-claims continue to exclude correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, and cross-toolchain determinism

#### Scenario: Contributor docs name THIR branch-control-flow boundaries [r[aggregate-valence-object-evidence-gate.scenario.thir-branch-control-flow-doc-boundaries]]

- GIVEN a contributor reads the contributor guide or current Valence evidence index
- WHEN they look for THIR selected fixture evidence
- THEN the docs name `branch-control-flow` as selected THIR fixture evidence
- AND they state selected THIR fixture evidence does not imply general THIR extraction, correctness, semantic equivalence, or MIR branch-control-flow support

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot records MUST preserve object-hash linkage, closed schemas/statuses, toolchain/extractor identity when available, and no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed selectors with positive and negative replay evidence; all other families or request shapes MUST remain `unsupported-planned`.

Supplement receipt checks MUST fail closed for malformed JSON, missing replay-critical fields, unsupported future schema versions, wrong receipt families, and boundary tampering while preserving no-correctness and no-semantic-equivalence boundaries.

#### Scenario: MIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.mir-fixture-inventory-selected-shapes]]

- GIVEN the rustc MIR fixture contract supports checked-in selector values `checked-arithmetic`, `call-return`, and `branch-control-flow`
- WHEN an operator requests fixture inventory for `--family mir --spike rustc-mir-fixture`
- THEN Octet lists exactly the selected MIR fixture shapes `checked-arithmetic`, `call-return`, and `branch-control-flow` in deterministic order
- AND each entry names its fixture path, function, support status, replay commands, and selected-fixture-only boundary

#### Scenario: MIR fixture inventory commands roundtrip selected fixture evidence [r[hir-mir-supplement-pilot.scenario.mir-inventory-commands-roundtrip]]

- GIVEN the MIR fixture inventory emits replay commands for each selected MIR fixture shape
- WHEN the emitted command pair for each inventory row is executed
- THEN `checked-arithmetic`, `call-return`, and `branch-control-flow` each emit `pilot-supported` MIR supplement evidence and replay successfully
- AND no inventory output accepts or implies non-selected MIR fixture shapes

#### Scenario: MIR branch-control-flow selected fixture is replayable [r[hir-mir-supplement-pilot.scenario.mir-branch-control-flow-roundtrip]]

- GIVEN the operator invokes `cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow`
- WHEN the generated receipt is checked with the matching MIR fixture selector
- THEN the check passes, the receipt names `mir-spike/branch_control_flow.rs::choose_branch`, and the boundary fields keep all correctness, semantic-equivalence, pure-cache, and cross-toolchain claims false.

#### Scenario: THIR fixture inventory lists selected branch-control-flow shape [r[hir-mir-supplement-pilot.scenario.thir-branch-control-flow-inventory]]

- GIVEN the rustc THIR fixture contract supports checked-in selector values `checked-arithmetic`, `call-return`, and `branch-control-flow`
- WHEN an operator requests fixture inventory for `--family thir --spike rustc-thir-fixture`
- THEN Octet lists exactly the selected THIR fixture shapes `checked-arithmetic`, `call-return`, and `branch-control-flow` in deterministic order
- AND each entry names its fixture path, function, support status, replay commands, and selected-fixture-only boundary

#### Scenario: THIR branch-control-flow commands roundtrip selected fixture evidence [r[hir-mir-supplement-pilot.scenario.thir-branch-control-flow-roundtrip]]

- GIVEN the THIR fixture inventory emits replay commands for `branch-control-flow`
- WHEN the emitted command pair for `branch-control-flow` is executed
- THEN it emits `pilot-supported` THIR supplement evidence and replays successfully
- AND the emitted supplement names `fixture_shape=branch-control-flow` while preserving no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain boundary fields

#### Scenario: Default MIR remains unsupported planned [r[hir-mir-supplement-pilot.scenario.default-mir-remains-unsupported-planned]]

- GIVEN the MIR supplemental evidence contract is emitted
- WHEN the contract is inspected
- THEN MIR remains `unsupported-planned` for default and non-fixture requests and does not claim general MIR extraction.

#### Scenario: Supplement compatibility accepts positive v1 receipt [r[hir-mir-supplement-pilot.scenario.supplement-compat-positive-v1]]

- GIVEN a generated selected-fixture supplement v1 receipt for a supported family and selector
- WHEN `cargo octet object supplement check --output-format json` replays the receipt with the matching selector
- THEN the check exits successfully with `status = "pass"`
- AND the report contains no typed errors

#### Scenario: Supplement compatibility rejects invalid receipts [r[hir-mir-supplement-pilot.scenario.supplement-compat-invalid-receipts]]

- GIVEN malformed, missing-field, unsupported-future-version, or wrong-family supplement receipt inputs
- WHEN `cargo octet object supplement check --output-format json` validates them
- THEN the command exits non-zero with `status = "invalid"`
- AND the report contains typed errors naming the failing kind and field while preserving non-proof boundary fields

#### Scenario: Supplement boundary tamper fails closed [r[hir-mir-supplement-pilot.scenario.supplement-boundary-tamper]]

- GIVEN a selected-fixture supplement v1 receipt whose boundary claims are tampered
- WHEN `cargo octet object supplement check --output-format json` replays it with the matching selector
- THEN the command exits non-zero with `status = "fail"`
- AND the comparison list names the `boundary` field as a mismatch

### Requirement: Rustc-backed HIR extractor spike [r[rustc-hir-extractor-spike]]

Octet MUST provide bounded, fixture-backed rustc HIR extractor spike evidence for selected Function Object v1 records while preserving the v1 object hash as the canonical review/provenance identity. Each supported HIR spike fixture shape MUST be checked in and explicitly selected with rustc/toolchain identity, extractor identity, schema version, fixture context, observation hash, and non-proof boundaries. Octet MUST expose closed fixture-shape inventory without accepting arbitrary HIR inputs.

#### Scenario: HIR fixture inventory lists closed selectors

- GIVEN the rustc HIR spike supports the checked-in fixture shapes `checked-arithmetic` and `branch-control-flow`
- WHEN an operator requests the HIR fixture inventory for `--spike rustc-hir-fixture`
- THEN Octet lists exactly the supported closed selector values
- AND each entry includes fixture shape, source path, function name, support status, and whether it is the default shape

#### Scenario: HIR fixture inventory preserves bounded support claims

- GIVEN an operator reads fixture inventory output in JSON or human format
- WHEN the output describes rustc HIR spike support
- THEN it names the support as selected-fixture-backed and toolchain-scoped
- AND it preserves no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain-stability boundary fields
- AND it does not claim arbitrary HIR, THIR, MIR, exhaustive Rust coverage, or semantic proof support

#### Scenario: HIR fixture inventory rejects unscoped requests

- GIVEN fixture inventory is only meaningful for the rustc HIR fixture spike
- WHEN an operator requests fixture inventory without `--family hir` and `--spike rustc-hir-fixture`
- THEN Octet rejects the request with a deterministic usage error
- AND no inventory output implies support for arbitrary supplement families or spikes

### Requirement: Object corpus fixture freshness rail

The system MUST expose a named object-corpus fixture freshness rail that verifies checked-in object-corpus fixtures still match freshly generated receipts and reports.

#### Scenario: Named fixture freshness rail passes

- GIVEN the checked-in internal corpus receipt, expanded internal corpus receipt, and expanded caveat taxonomy fixtures
- WHEN an operator runs `nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L`
- THEN the rail runs the focused fresh-fixture parity tests and fails if any checked-in fixture is stale.

#### Scenario: Aggregate Valence gate names freshness rail

- GIVEN the aggregate `valence-object-evidence` gate
- WHEN it succeeds
- THEN it requires the named object-corpus fixture freshness rail and prints `valence_object_evidence.object_corpus_fixture_freshness_gate: PASS`.

#### Scenario: Guidance names freshness obligation

- GIVEN contributor and OpenSpec task-template guidance for Valence object-evidence work
- WHEN a change touches object-corpus inputs such as `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, THIR/MIR fixture inventories, or object-corpus fixture paths
- THEN the guidance names the fixture regeneration obligation and the named freshness rail before closeout.
