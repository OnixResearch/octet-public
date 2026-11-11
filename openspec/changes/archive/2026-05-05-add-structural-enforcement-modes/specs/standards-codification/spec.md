## ADDED Requirements

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

#### Scenario: Crate boundary contract exports core and shell obligations [r[nickel-mechanism-contracts.scenario.crate-boundary-contract]]

- GIVEN a policy uses `mechanism.kind = 'crate_boundary`
- WHEN the Nickel export is generated
- THEN the exported mechanism includes core crates, shell crates, no-std requirement, alloc allowance, forbidden core imports, forbidden core dependencies, and optional compatibility profile

#### Scenario: Capability contract exports constructor authority [r[nickel-mechanism-contracts.scenario.capability-contract]]

- GIVEN a policy uses `mechanism.kind = 'capability_api`
- WHEN the Nickel export is generated
- THEN the exported mechanism includes capability type names, approved constructor modules, forbidden constructor scopes, allowed injected value inputs, private-field requirement, and sealed-trait requirement

#### Scenario: Bounded and type-state contracts export API obligations [r[nickel-mechanism-contracts.scenario.bounded-typestate-contracts]]

- GIVEN policies use `mechanism.kind = 'bounded_primitive` and `mechanism.kind = 'typestate`
- WHEN the Nickel export is generated
- THEN bounded primitive exports include approved types, forbidden APIs, and capacity-witness requirements
- AND type-state exports include state names, transitions, dangerous operations, required states, and raw-primitive API restrictions

#### Scenario: Seed corpus contract exports provenance and role [r[nickel-mechanism-contracts.scenario.seed-corpus-contract]]

- GIVEN a policy uses `mechanism.kind = 'seed_corpus`
- WHEN the Nickel export is generated
- THEN the exported seed contract includes source repo, source commit, selected paths, role, license review, provenance review, hash algorithm, hashes, local acceptance command, and expected signal
- AND proof-relevant seeds require local proof or an explicit trust-boundary report before acceptance

### Requirement: Structural Mechanism Validation [r[structural-mechanism-validation]]

The system MUST validate claimed structural enforcement mechanisms before accepting a policy as structurally enforced.

Structural validation MUST cover at least no-std core crates, shell-only side effects, approved capability constructors, bounded primitive use, type-state protocol surfaces, and crate dependency direction.

#### Scenario: No-std core boundary is enforced [r[structural-mechanism-validation.scenario.no-std-core-boundary]]

- GIVEN a project declares a crate as a Tiger Style deterministic core
- WHEN structural validation runs
- THEN the checker fails unless the crate is `#![no_std]` or has an explicitly justified compatibility profile
- AND the core crate is rejected if it imports shell-only effects such as filesystem, network, environment, wall-clock time, async runtimes, or process execution

#### Scenario: Shell-to-core dependency direction is enforced [r[structural-mechanism-validation.scenario.shell-core-dependency-direction]]

- GIVEN a project declares core crates and shell crates
- WHEN crate graph validation runs
- THEN shell crates may depend on core crates
- AND core crates MUST NOT depend on shell crates or effect adapter crates

#### Scenario: Capability constructors are shell-owned [r[structural-mechanism-validation.scenario.capability-constructors-shell-owned]]

- GIVEN a policy claims time, I/O, randomness, or authority is enforced by capability APIs
- WHEN source validation scans constructors and public APIs
- THEN capability construction is accepted only in approved shell modules or private wrappers
- AND core logic receives injected values, snapshots, effect descriptions, or sealed capability types rather than reaching for ambient effects

#### Scenario: Type-state protocol surfaces reject raw operations [r[structural-mechanism-validation.scenario.typestate-rejects-raw-operations]]

- GIVEN a protocol declares raw, parsed, validated, authorized, or committed states
- WHEN API validation runs
- THEN dangerous operations accept only the refined state type required by the policy
- AND public APIs exposing raw primitive substitutes for the refined state are reported unless explicitly justified

#### Scenario: Bounded primitive requirement rejects unbounded replacement [r[structural-mechanism-validation.scenario.bounded-primitive-required]]

- GIVEN a policy declares bounded collections, queues, retries, or buffers as structural
- WHEN source validation checks the configured scope
- THEN approved bounded primitive types or explicit bound witnesses are accepted
- AND unbounded growth APIs remain findings even when ordinary lint diagnostics are otherwise clean

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]

The system MUST require proof-rail evidence when a policy declares `current = "verified"` or targets verified enforcement for critical primitives.

Verified enforcement MUST name the proof backend, source/proof mapping, expected command, accepted proof artifact shape, and trust-boundary accounting.

#### Scenario: Verified policy requires proof command [r[verified-enforcement-rails.scenario.proof-command-required]]

- GIVEN a policy declares verified enforcement for a module, primitive, or protocol
- WHEN validation runs
- THEN validation fails unless the policy names a runnable proof command and the source/proof paths it covers
- AND the saved evidence contract names the expected pass signal and any allowed trust-boundary count

#### Scenario: Proof-relevant code cannot be imported as trusted by default [r[verified-enforcement-rails.scenario.import-not-trusted-by-default]]

- GIVEN code or examples are imported from a sibling repository with existing proofs
- WHEN Tigerstyle accepts the vendored seed
- THEN the imported material is treated as seed corpus or reference implementation until local proof, lint, and fixture checks pass in this repository
- AND the source commit, hashes, and local verification output are recorded

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
