# SpaceWasm MVP Artifact Profile Specification

## Purpose

Provide deterministic execution-free facts and policy decisions for exact WebAssembly core-module bytes against one reviewed SpaceWasm MVP cohort without implying runtime acceptance, correctness, safety, certification, or release eligibility.

## Requirements

### Requirement: The SpaceWasm artifact profile is typed and reviewable

r[octet.spacewasm_mvp.profile] Octet MUST define a typed Nickel `spacewasm-mvp` profile with declared SpaceWasm cohort, module kind, proposal posture, imports/exports, ABI, memories/tables/start/growth rules, structural/resource limits, severity, and non-claims and MUST consume a deterministic fresh export.

#### Scenario: Valid profile is exported
- GIVEN the Nickel profile satisfies its contracts and generated runtime input is fresh
- WHEN profile loading runs
- THEN Octet MAY evaluate artifacts under one deterministic profile identity.

#### Scenario: Profile is invalid or stale
- GIVEN required fields are absent, contradictory, or differ from the generated export
- WHEN profile loading runs
- THEN Octet MUST fail closed before artifact decisions are accepted.

### Requirement: One exact SpaceWasm cohort defines support intent

r[octet.spacewasm_mvp.cohort] The profile MUST bind an exact reviewed SpaceWasm source/materialization identity and checked support-matrix projection, and any source, lock, feature, or projection change MUST produce a new cohort identity and require complete fixture replay.

#### Scenario: Cohort and support projection match
- GIVEN source/materialization and support facts match the declared review
- WHEN cohort verification runs
- THEN Octet MAY report profile decisions under that cohort.

#### Scenario: Upstream or projection drifts
- GIVEN the source, lockfile, support matrix, or generated projection differs
- WHEN verification runs
- THEN Octet MUST report stale-cohort/profile diagnostics and MUST NOT widen feature admission.

### Requirement: Artifact validation remains execution-free

r[octet.spacewasm_mvp.validation] Octet MUST use the shared wasm-tools analysis core to classify exact bytes as core module or other kind and evaluate proposals, imports/exports, ABI, memories/tables, start, growth, structural limits, and producer metadata without invoking SpaceWasm or another runtime.

#### Scenario: Core module satisfies the profile
- GIVEN exact bytes parse and every checked fact satisfies the declared profile
- WHEN analysis runs
- THEN Octet MAY emit a static profile pass with all extracted facts.

#### Scenario: Artifact violates the profile
- GIVEN bytes are malformed, a component, require WIT/WASI, use a disallowed proposal, or violate another declared rule
- WHEN analysis runs
- THEN Octet MUST emit the corresponding deterministic finding and deny profile conformance.

### Requirement: Resource facts distinguish artifact declarations from runtime configuration

r[octet.spacewasm_mvp.resources] Octet MUST evaluate statically observable memory/table/function/local/start/growth and structural resource facts against named profile fields and MUST label interpreter allocator, code-page, stack, and instruction settings as external runtime configuration when they are not encoded in the artifact.

#### Scenario: Declared resources fit
- GIVEN every observable declaration stays within profile limits
- WHEN resource policy runs
- THEN Octet MAY report static resource conformance.

#### Scenario: Report infers hidden runtime capacity
- GIVEN a report claims that static declarations prove configured SpaceWasm allocator, stack, code, or execution capacity
- WHEN report validation runs
- THEN Octet MUST reject the unsupported inference.

### Requirement: Artifact, cohort, and interface identities remain distinct

r[octet.spacewasm_mvp.identity] Octet MUST compute BLAKE3 identity over exact original artifact bytes and MUST keep SpaceWasm cohort, policy profile, and optional normalized ABI/interface identities in separate typed domains.

#### Scenario: Same ABI has different bytes
- GIVEN two artifacts normalize to the same declared ABI but have different bytes
- WHEN identities are computed
- THEN their artifact identities MUST differ and neither interface identity may replace them.

#### Scenario: Identity roles are confused
- GIVEN a report substitutes cohort, profile, or normalized ABI identity for exact artifact identity
- WHEN verification runs
- THEN Octet MUST reject the report.

### Requirement: Findings and handoffs are stable and verifiable

r[octet.spacewasm_mvp.findings] Octet MUST emit ordered structured findings and a verifiable report bundle binding exact bytes, profile/cohort, extracted facts, decisions, diagnostics, scopes, and non-claims.

#### Scenario: Bundle is verified independently
- GIVEN a complete report bundle and exact artifact bytes
- WHEN artifact verification runs
- THEN every member, BLAKE3, parent edge, profile/cohort, and finding summary MUST match.

#### Scenario: Report or artifact is tampered
- GIVEN any byte, identity, fact, decision, finding, or parent edge differs
- WHEN verification runs
- THEN the bundle MUST fail closed.

### Requirement: Mantle handoff does not transfer build semantics

r[octet.spacewasm_mvp.handoff] Octet MUST provide a stable exact-byte request/report contract for standalone and Mantle-orchestrated checks and MUST NOT trust a store path, build result, materialization label, or producer summary as artifact facts.

#### Scenario: Mantle supplies exact final bytes
- GIVEN Mantle passes exact bytes and declared profile/cohort identities
- WHEN Octet evaluates them
- THEN the report MUST bind Octet's independent artifact BLAKE3 and decisions.

#### Scenario: Handoff facts do not match bytes
- GIVEN producer labels or expected digests differ from Octet's measurement
- WHEN handoff validation runs
- THEN Octet MUST reject the request or report the mismatch without importing build trust.

### Requirement: Runtime reports remain recorded-only external evidence

r[octet.spacewasm_mvp.external_evidence] Octet MAY ingest identity-verified Mantle, ChaosControl, or Molten SpaceWasm reports as recorded-only evidence but MUST keep producer semantics, runtime observations, static decisions, and claim roles separate.

#### Scenario: External report links correctly
- GIVEN producer, artifact, cohort, profile, result, role, and non-claims verify
- WHEN external evidence is linked
- THEN Octet MAY retain the report reference without changing its static decision.

#### Scenario: Runtime success conflicts with static policy
- GIVEN an external runtime report says execution succeeded for bytes denied by the profile
- WHEN evidence is assembled
- THEN Octet MUST preserve both facts and MUST NOT convert runtime success into static conformance.

### Requirement: SpaceWasm findings export bounded Valence roles

r[octet.spacewasm_mvp.valence_export] Octet MUST export verified SpaceWasm artifact facts only as recorded-only or boundary evidence unless a separate checked property profile explicitly authorizes a narrower role.

#### Scenario: Static pass is exported
- GIVEN a verified profile report
- WHEN Valence export runs
- THEN the export MUST bind exact artifact/profile/cohort/finding identities and explicit execution/safety/certification/release non-claims.

### Requirement: SpaceWasm artifact evidence rejects overclaims

r[octet.spacewasm_mvp.nonclaims] Octet MUST reject reports that infer SpaceWasm decode or execution success, memory safety, semantic equivalence, sandbox effectiveness, flight qualification, production readiness, or release eligibility from static profile conformance.

#### Scenario: Static pass requests runtime authority
- GIVEN a passing artifact report is labeled execution, safety, certification, sandbox, or release proof
- WHEN claim validation runs
- THEN Octet MUST reject the unsupported promotion.

### Requirement: The profile has deterministic positive and negative corpora

r[octet.spacewasm_mvp.fixtures] The profile MUST include positive admitted MVP/core-ABI/resource fixtures and negative malformed, component, WIT/WASI, proposal, import, ABI, memory/table/growth, structural-limit, cohort, profile, identity, handoff, external-evidence, and overclaim fixtures with exact generation and expected issue facts.

#### Scenario: Generated case is replayed
- GIVEN a checked seed/configuration/transform manifest
- WHEN bounded regeneration runs
- THEN exact artifact bytes and expected issue class MUST match.

### Requirement: Profile decisions have a functional core

r[octet.spacewasm_mvp.functional_core] Profile/support validation, byte-fact policy evaluation, identity material, finding order, external-evidence DTO validation, and report construction MUST be pure deterministic logic.

#### Scenario: Identical inputs are evaluated twice
- GIVEN identical artifact bytes, profile, cohort, and optional external facts
- WHEN the core evaluates them
- THEN it MUST produce identical identities, decisions, findings, and diagnostics without filesystem, process, runtime, network, clock, environment, or output effects.

### Requirement: Final validation is complete and bounded

r[octet.spacewasm_mvp.final_validation] Changes to the profile MUST run focused core/CLI tests, profile/export freshness, positive and negative corpora, standalone/Mantle handoffs, artifact verification, external-evidence and Valence fixtures, Cairn validation, lifecycle gates, and the smallest relevant Octet checks.

#### Scenario: A required rail is unavailable
- GIVEN a dependency change or external fixture cannot run
- WHEN closeout evidence is prepared
- THEN it MUST record the exact blocker, affected claim, and next-best check without reporting the rail passed.
