# Wasm Component Artifact Rail Specification

## Purpose

Provide deterministic wasm-tools-backed module and component artifact facts for Octet consumers without executing artifacts or promoting interface compatibility into semantic, authority, or release claims.

## Requirements

### Requirement: Wasm artifact profiles are typed and reviewable

r[octet.wasm_component.profile] Octet MUST define Wasm artifact profiles in typed Nickel with declared artifact kinds, WIT worlds, feature posture, import/export policy, memory/table/growth rules, metadata policy, resource declarations, severity, and non-claims, and MUST consume a deterministic checked export at runtime.

#### Scenario: Valid profile is exported
- GIVEN a complete Nickel profile satisfies its contract
- WHEN the profile export runs
- THEN Octet MUST produce deterministic runtime input and MUST bind its BLAKE3 identity into artifact evidence.

#### Scenario: Profile is malformed or stale
- GIVEN a profile has an unknown field, unsupported proposal, duplicate rule, invalid world, or stale generated export
- WHEN policy validation runs
- THEN Octet MUST fail closed before artifact decisions are accepted.

### Requirement: One wasm-tools cohort defines parser identity

r[octet.wasm_component.cohort] Octet MUST pin and identify one compatible wasm-tools crate cohort for Wasm validation, WIT/component parsing, metadata inspection, encoding, generation, mutation, and shrinking.

#### Scenario: Cohort is upgraded
- GIVEN any pinned wasm-tools cohort member changes
- WHEN the rail is rebuilt
- THEN Octet MUST produce a new cohort identity and MUST rerun all positive and negative conformance fixtures.

### Requirement: Artifact validation is execution-free

r[octet.wasm_component.validation] Octet MUST classify and validate modules/components, proposals, WIT worlds, imports/exports, WASI/host imports, memories/tables/growth, entrypoint posture, producer metadata, and declared resource shape from bytes without executing the artifact.

#### Scenario: Valid component satisfies profile
- GIVEN component bytes are valid and every extracted fact satisfies the selected profile
- WHEN artifact validation runs
- THEN Octet MUST emit a passing bounded-facts report.

#### Scenario: Artifact requires disallowed capability or feature
- GIVEN a valid artifact imports undeclared WASI/host functionality or enables a proposal denied by the profile
- WHEN policy evaluation runs
- THEN Octet MUST emit a deterministic finding and apply the declared severity without invoking the artifact.

### Requirement: Exact bytes and normalized interfaces keep separate identities

r[octet.wasm_component.identity] Octet MUST compute exact artifact BLAKE3 over original bytes and MAY compute a separate normalized WIT/interface BLAKE3, but MUST NOT substitute one role for the other or infer behavioral equivalence from an interface match.

#### Scenario: Different components implement one world
- GIVEN two different component byte strings implement the same normalized WIT world
- WHEN identities are constructed
- THEN their exact artifact identities MUST differ and their interface identities MAY match without an equivalence claim.

#### Scenario: Identity roles are confused
- GIVEN a report places an interface hash in the exact-byte role or claims matching interfaces identify one implementation
- WHEN report validation runs
- THEN Octet MUST reject the report.

### Requirement: Findings and bundles are structured and verifiable

r[octet.wasm_component.findings] Octet MUST emit ordered structured findings and a verifiable artifact bundle that binds profile, cohort, input bytes, extracted facts, policy decisions, diagnostics, scopes, and explicit non-claims.

#### Scenario: Bundle is verified
- GIVEN a completed Wasm artifact bundle
- WHEN `cargo octet artifact verify` evaluates it
- THEN every required file, parent link, BLAKE3 digest, profile/cohort identity, and finding summary MUST match.

#### Scenario: Finding or summary is tampered
- GIVEN a finding, extracted fact, digest, summary count, or profile link changes after collection
- WHEN bundle verification runs
- THEN verification MUST fail closed.

### Requirement: Mantle can orchestrate the rail without owning its semantics

r[octet.wasm_component.mantle_handoff] Octet MUST provide a stable exact-byte CLI/report contract that lets Mantle invoke a declared artifact profile after build-local validation and bind the resulting artifact BLAKE3, Octet profile, cohort, findings, and report identity into a materialization bundle; Octet MUST remain independently runnable and MUST NOT trust store paths or Mantle success as artifact facts.

#### Scenario: Mantle checks exact final bytes
- GIVEN Mantle supplies final portable bytes and a supported Octet profile
- WHEN the artifact rail runs
- THEN Octet MUST analyze those exact bytes and return the same verifiable bundle a standalone invocation would produce.

#### Scenario: Mantle binds a report to different bytes
- GIVEN a materialization request pairs an Octet report with a different artifact, profile, or cohort identity
- WHEN handoff verification runs
- THEN Octet MUST reject the mismatch without interpreting build or release policy.

### Requirement: The rail has deterministic positive and negative corpora

r[octet.wasm_component.fixtures] Every promoted Wasm artifact check MUST have positive and negative fixtures, and generated/mutated fixtures MUST bind generator or mutator identity, seed, input, transform sequence, expected issue class, and optional shrink predicate.

#### Scenario: Generated failure is reproduced
- GIVEN a bounded generator or mutator case exposes an expected issue
- WHEN the saved manifest is replayed
- THEN Octet MUST reproduce the same artifact or failure class and MAY shrink it under the bound predicate.

#### Scenario: Check has only a positive fixture
- GIVEN a proposed blocking check has no fixture proving a near-miss or invalid case fails correctly
- WHEN promotion is evaluated
- THEN the check MUST remain advisory.

### Requirement: Artifact decisions have a functional core

r[octet.wasm_component.functional_core] Byte parsing, fact normalization, profile validation, policy evaluation, finding ordering, identity material construction, fixture expectation evaluation, and report DTO construction MUST be pure deterministic logic.

#### Scenario: Identical bytes and profile produce identical report
- GIVEN identical artifact bytes, profile DTO, cohort facts, and expected WIT facts
- WHEN the analysis core runs
- THEN it MUST return the same report without filesystem, environment, process, clock, network, runtime execution, or output effects.

### Requirement: Valence export retains bounded verification roles

r[octet.wasm_component.valence_export] Octet MUST export verified Wasm artifact facts to Valence only as recorded-only or boundary evidence unless a separate checked property profile explicitly narrows the claim.

#### Scenario: Artifact facts are exported
- GIVEN an Octet artifact bundle passes verification
- WHEN a Valence adapter export is requested
- THEN the export MUST retain artifact/interface role separation, profile/cohort identity, findings, spans or binary offsets where available, and non-claims.

### Requirement: Artifact evidence rejects overclaims

r[octet.wasm_component.nonclaims] Octet MUST reject Wasm artifact evidence that claims execution correctness, sandbox effectiveness, capability authority, provenance truth, semantic equivalence, release eligibility, or whole-system safety from static artifact validation.

#### Scenario: Valid component requests release proof role
- GIVEN a statically valid component report requests a correctness or release-eligibility role
- WHEN evidence validation runs
- THEN Octet MUST reject the unsupported promotion.

### Requirement: Wasm artifact rail validation is complete

r[octet.wasm_component.final_validation] Changes to the Wasm artifact rail MUST run focused core/CLI tests, standalone/Mantle handoff tests, positive and negative corpus checks, artifact verification, policy freshness checks, Valence export fixtures, Cairn gates, and the smallest relevant Octet quality checks.

#### Scenario: Rail change is reviewed
- GIVEN profile, parser, finding, fixture, bundle, or export behavior changes
- WHEN validation evidence is assembled
- THEN it MUST identify the exact profile and cohort and include both positive and negative results.
