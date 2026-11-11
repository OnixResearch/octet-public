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

#### Scenario: Status or registry binding disagrees with the receipt
- GIVEN a receipt whose status differs from the status its recorded findings imply, or whose config identity differs from its recorded registry identity
- WHEN bundle verification runs
- THEN verification MUST fail closed even when the summary and provenance digests were regenerated.

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

### Requirement: Molten profile admits only declared shape

r[octet.molten_profile.profile] Octet MUST define a typed Nickel `molten-component-baseline` profile that admits component kind, the single declared `component::preserves-payload::Type(Eq(0))` import, bulk memory and padded indirect calls, baseline structural and resource limits, error severity, recorded-only verification role, and explicit non-claims, and MUST consume its deterministic fresh export.

#### Scenario: Admitted bundle passes
- GIVEN the supported Molten component with that type import and bulk memory
- WHEN the profile analyzes exact bytes
- THEN no disallowed-import or disallowed-proposal finding is produced

#### Scenario: Any other import denies
- GIVEN a component importing anything else
- WHEN the profile analyzes it
- THEN the import is a disallowed-import finding

### Requirement: Bulk memory is a granular proposal

r[octet.molten_profile.granular_proposal] The `bulk-memory` proposal token MUST map to the bulk-memory validation feature only, without enabling other Wasm 2 proposals, and unknown proposal tokens MUST fail profile validation.

#### Scenario: Unknown proposal denies
- GIVEN a profile naming `bulk-memory-unknown`
- WHEN registry validation runs
- THEN it reports an unknown proposal

### Requirement: Padded indirect calls do not admit reference types

r[octet.molten_profile.indirect_encoding] The `call-indirect-overlong` token MUST map only to the padded indirect-call table-immediate encoding feature, without enabling reference types, SIMD or the rest of Wasm 2. The Molten profile MUST explicitly declare this encoding; the portable profile MUST remain unchanged.

#### Scenario: Linked zero table index validates
- GIVEN an otherwise valid module with an overlong LEB zero table immediate
- WHEN the Molten feature policy validates its exact bytes
- THEN it passes while the same policy without that feature denies it.

#### Scenario: Unrelated proposal still denies
- GIVEN a module containing reference-type or SIMD instructions
- WHEN the Molten feature policy validates it
- THEN validation denies it.

### Requirement: Validate exact policy boundaries

r[octet.molten_profile.validation] Octet MUST exercise positive bulk-memory and padded indirect-call byte validation alongside negative portable-policy, unknown-token, reference-type and SIMD cases.

#### Scenario: Boundary corpus executes
- GIVEN the checked-in profile registry and exact-byte boundary corpus
- WHEN focused Wasm artifact tests execute
- THEN the declared features pass and all unrelated or unknown cases deny.

### Requirement: Baseline stays import-free

r[octet.molten_profile.baseline_unchanged] `portable-component-baseline` MUST keep empty allowed imports and only `wasm1` and `component-model` proposals.

#### Scenario: Baseline is unchanged
- GIVEN the existing baseline profile
- WHEN the registry is validated
- THEN it still admits no imports and no bulk memory

### Requirement: Non-claims remain explicit

r[octet.molten_profile.nonclaims] Profile admission of the declared import, bulk memory and padded indirect calls MUST NOT be presented as admitting other imports, proposals, runtime authority, execution correctness, or release eligibility.

#### Scenario: Evidence carries the added non-claim
- GIVEN a receipt produced with `molten-component-baseline`
- WHEN it is reviewed
- THEN the profile's added admission non-claim is present

### Requirement: Metadata scope follows binary nesting

r[octet.molten_effects.metadata_scope] Duplicate custom-section policy MUST compare section names only within the same module/component binary scope and MUST retain deterministic scope-qualified metadata facts.

#### Scenario: Independent nested name sections pass
- GIVEN separate nested binaries each containing one name section
- WHEN a reject-duplicates profile analyzes the component
- THEN no duplicate metadata finding occurs

#### Scenario: Same-scope name duplication denies
- GIVEN a nested binary containing two name sections
- WHEN that profile analyzes the component
- THEN duplicate-custom-section is reported

### Requirement: Internal binding does not erase unresolved authority

r[octet.molten_effects.boundary] Octet MUST distinguish root exports and unresolved imports from imports proven supplied by validated local instantiation, MUST record bound imports separately, and MUST retain unresolved authority from unused or exported reusable definitions. A nested unresolved component import MUST NOT inherit root import allowances.

#### Scenario: Internally supplied adapter passes
- GIVEN a valid local provider instance supplying an adapter's function and table imports
- WHEN an import-free profile analyzes the component
- THEN those adapter imports are recorded as bound and do not become external import denials

#### Scenario: Unused or exported adapter retains imports
- GIVEN the same adapter uninstantiated or exported as a reusable module
- WHEN an import-free profile analyzes the component
- THEN its unresolved imports are denied

### Requirement: Resource evidence counts definitions and instantiations

r[octet.molten_effects.resources] Resource counts MUST exclude proven bound import aliases as independent allocations, MUST multiply definitions by local and enclosing instantiation multiplicity with saturating arithmetic, MUST conservatively count unused definitions once, and MUST enforce retained declaration bounds. Unknown imported/aliased generative implementations MUST fail accounting closed. Absence of a declared maximum MUST NOT be reported as a runtime bound.

#### Scenario: Table alias is not a new table
- GIVEN one defined table supplied to an adapter by import
- WHEN the component is analyzed
- THEN the independent table count is one

#### Scenario: Repeated instantiation exceeds the count bound
- GIVEN a one-table module instantiated twice and a one-table count limit
- WHEN the component is analyzed
- THEN the count is two and a resource-limit finding is reported

#### Scenario: Effects table boundaries deny above limits
- GIVEN the effect profile with two independent tables allowed and the declared element limit
- WHEN table count or declaration elements exceed the respective limit
- THEN resource-limit or table-limit is reported, while the exact boundary is admitted

### Requirement: Effects profile admits exact structural surface only

r[octet.molten_effects.profile] Nickel MUST define separate `molten-component-effects-v1` with `component_interface = "structural-v1"`, exactly required root preserves-payload equality(list<u8>) type bookkeeping and effect-request function imports, and sole required invoke export. Both functions MUST have synchronous `(list<u8>) -> result<list<u8>,string>` signatures. The profile MUST allow only wasm1/component-model/bulk-memory/call-indirect-overlong, two imports, one export, one memory and two independent tables, retain remaining baseline limits and denial postures, and remain recorded-only. Both existing published profile policies MUST remain unchanged.

#### Scenario: Padded indirect encoding stays narrowly scoped
- GIVEN canonical or overlong LEB table-zero call-indirect encoding
- WHEN the effect profile analyzes it
- THEN that encoding is admitted without enabling general reference types or SIMD
- AND undeclared nonzero table indices and reference operations are denied

#### Scenario: Canonical type indexing is immaterial
- GIVEN identical admitted structural signatures with different local type indices
- WHEN the exact bytes are analyzed
- THEN both pass with equal interface identities and different exact-byte identities

#### Scenario: Wrong or undeclared boundary denies
- GIVEN missing required imports/exports, extra WASI/product functions, or wrong argument/result/payload types
- WHEN the effects profile analyzes the component
- THEN disallowed-import, disallowed-export, or abi-mismatch findings deny it

#### Scenario: Existing profiles remain closed
- GIVEN the effect surface with its product function import
- WHEN either existing baseline profile analyzes it
- THEN the effect-request import is denied

### Requirement: Identity and authority remain separate

r[octet.molten_effects.identity] Receipts MUST retain exact original byte, selected canonical profile, full registry, and cohort identities. Corrected boundary interface identities MUST use the v2 domain. Optional profile additions MUST preserve pre-existing profile DTO identities when absent. Structural surface admission MUST NOT authenticate a package/world name or transfer canonical Preserves, effect/capability, semantic, runtime, provenance, or release authority to Octet.

#### Scenario: Parent replays the real guest and receipts
- GIVEN the retained normalized supported effect guest and generated profile registry
- WHEN the parent integrated verification analyzes the exact bytes and verifies the artifact bundle
- THEN the new profile passes static analysis with the exact bytes/profile identities retained and explicit recorded-only non-claims
- AND no runtime execution or semantic correctness claim follows from that pass
