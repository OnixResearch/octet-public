## ADDED Requirements

### Requirement: Octet compiler-policy provider manifest

r[octet.compiler_policy_provider.manifest] Octet MUST expose a deterministic compiler-policy provider manifest that downstream build systems can consume without importing Octet lint implementation internals.

#### Scenario: Manifest identifies adapter artifacts

r[octet.compiler_policy_provider.manifest.identity]

- GIVEN Octet exports a compiler-policy provider manifest
- WHEN a downstream build system reads the manifest
- THEN the manifest MUST identify the provider name, schema version, profile name, adapter kind, compatible rustc/toolchain identity, Dylint driver identity, Octet lint-library identity, lint config identity, standards policy identity when selected, and required invocation environment.
- AND every local artifact identity intended for receipt binding MUST include a BLAKE3 digest or an equivalent derivation/source identity that downstream tools can hash deterministically.

#### Scenario: Manifest export is downstream-consumable

r[octet.compiler_policy_provider.manifest.export]

- GIVEN an operator requests the Octet compiler-policy provider manifest
- WHEN Octet renders the manifest through its command or Nix app surface
- THEN the output MUST be deterministic for equivalent inputs.
- AND the output MUST include enough information for Mantle or another build orchestrator to invoke the adapter, verify compatibility, and bind provider identity into artifact receipts.

#### Scenario: Invalid manifest material fails closed

r[octet.compiler_policy_provider.manifest.fail_closed]

- GIVEN manifest material omits required rustc identity, driver identity, lint-library identity, config identity, selected standards identity, or valid digest syntax
- WHEN Octet validates the provider manifest
- THEN validation MUST fail with deterministic diagnostics before the manifest can be used as an artifact-acceptance profile.

### Requirement: Artifact-acceptance policy profile

r[octet.compiler_policy_provider.profile] Octet MUST define at least one artifact-acceptance profile for downstream builds that distinguishes audit evidence from fail-closed enforcement.

#### Scenario: Required profile blocks hidden lint disablement

r[octet.compiler_policy_provider.profile.required_lints]

- GIVEN a profile is intended for downstream artifact acceptance
- WHEN the profile marks a lint or standards policy as required
- THEN Octet MUST reject hidden disablement of that required check through config, command-line weakening, or unstructured source-level allows.
- AND accepted exceptions MUST pass the structured waiver contract.

#### Scenario: Profile names bounded claims

r[octet.compiler_policy_provider.profile.non_claims]

- GIVEN Octet exports an artifact-acceptance profile
- WHEN downstream evidence cites that profile
- THEN the profile MUST state that passing evidence supports configured-profile compliance for the selected source scope.
- AND the profile MUST NOT authorize claims of program correctness, complete FCIS architecture proof, full safety, or absence of all defects without separate evidence.

### Requirement: Structured policy waivers

r[octet.compiler_policy_provider.waivers] Octet MUST provide a structured waiver contract for accepted policy exceptions and MUST make waiver summaries exportable for downstream receipts.

#### Scenario: Waiver metadata is required

r[octet.compiler_policy_provider.waivers.validation]

- GIVEN code suppresses a required Octet lint or standards policy
- WHEN Octet validates the suppression under an artifact-acceptance profile
- THEN the suppression MUST include a policy identifier, source location or narrow scope, owner, rationale, review trigger, and safer default.
- AND missing or malformed waiver metadata MUST fail closed.

#### Scenario: Waivers are receipt-friendly

r[octet.compiler_policy_provider.waivers.summary]

- GIVEN a build or standards check accepts structured waivers
- WHEN Octet emits machine-readable output for downstream receipt binding
- THEN the output MUST include a deterministic waiver summary with policy identifiers, source scopes, owners, and review triggers.
- AND the output MUST avoid secret material and avoid relying on ambient absolute temporary paths as identity.

### Requirement: Downstream build-system boundary

r[octet.compiler_policy_provider.downstream_boundary] Octet MUST keep rule authority in Octet while allowing downstream build systems to own compiler invocation, artifact acceptance, cache identity, and release receipts.

#### Scenario: Mantle consumes provider identity without owning rules

r[octet.compiler_policy_provider.downstream_boundary.mantle]

- GIVEN Mantle consumes an Octet compiler-policy provider manifest
- WHEN Mantle compiles a Rust unit under an Octet-required mode
- THEN Octet MUST remain the authority for lint semantics, standards-policy semantics, config validation, and waiver validation.
- AND Mantle MUST be able to treat the manifest and generated policy outputs as external receipt-bound inputs rather than as linked Octet implementation code.

#### Scenario: Compiler and standards rails stay distinct

r[octet.compiler_policy_provider.downstream_boundary.rails]

- GIVEN a provider profile includes both Dylint compiler lints and `octet-standards` FCIS source-shape checks
- WHEN Octet exports the profile
- THEN the manifest MUST identify the compiler-lint rail and standards rail separately.
- AND passing one rail MUST NOT be described as passing the other rail.
