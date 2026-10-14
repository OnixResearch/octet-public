# Dynamic Evidence Rails Specification

## Purpose

Defines the `dynamic-evidence-rails` capability.

## Requirements

### Requirement: Dynamic evidence profile registry

r[octet.dynamic_evidence.profile_registry] Octet SHOULD define a local registry of opt-in dynamic evidence profiles for high-risk scopes such as unsafe boundaries, FFI, concurrency, heap growth, and cache or hot-path review.

#### Scenario: Profile declares tool and acceptance signal

r[octet.dynamic_evidence.profile_registry.profile_fields]

- GIVEN a dynamic evidence profile is configured
- WHEN Octet validates the profile
- THEN the profile SHOULD declare profile ID, tool kind, checked scope, allowed command shape, fixture or workload description, expected acceptance signal, evidence tier, and manual-review boundary.

#### Scenario: Starter check does not require profiles

r[octet.dynamic_evidence.profile_registry.not_default]

- GIVEN a consumer runs starter `cargo octet check`
- WHEN no dynamic evidence profile is selected
- THEN Octet MUST NOT require Miri, Loom, sanitizers, Valgrind, heap profilers, or cache profilers to be installed.

### Requirement: Dynamic evidence runner

r[octet.dynamic_evidence.runner] Octet SHOULD provide an opt-in runner that executes a selected dynamic evidence profile and records the result without broadening the claim beyond the selected scope.

#### Scenario: Selected profile writes receipt

r[octet.dynamic_evidence.runner.receipt_written]

- GIVEN an operator selects a configured dynamic evidence profile
- WHEN Octet runs the profile command
- THEN Octet SHOULD record tool identity, version when available, command argv, working directory, fixture or workload, exit status, acceptance signal, observed result, and bounded log identity.

#### Scenario: Unconfigured command is rejected

r[octet.dynamic_evidence.runner.rejects_unconfigured]

- GIVEN a user requests a dynamic evidence command not declared by an accepted profile
- WHEN Octet resolves the request
- THEN Octet MUST reject it with a diagnostic naming the unknown profile or disallowed command shape.

### Requirement: Dynamic evidence tier boundaries

r[octet.dynamic_evidence.tier_boundary] Dynamic evidence receipts MUST preserve evidence-tier boundaries and MUST NOT claim formal proof unless backed by a separate validated formal-proof receipt.

#### Scenario: Miri and Loom stay scoped validation

r[octet.dynamic_evidence.tier_boundary.scoped]

- GIVEN a Miri or Loom profile passes for a fixture
- WHEN Octet renders the receipt
- THEN the receipt SHOULD use `transcript_assertion`, `manual_review_required`, or another explicit non-proof tier.
- AND it MUST NOT claim whole-program correctness, total unsafe-code soundness, or exhaustive concurrency coverage.

#### Scenario: Failed dynamic evidence remains reviewable

r[octet.dynamic_evidence.tier_boundary.failed]

- GIVEN a dynamic evidence profile fails or produces inconclusive output
- WHEN Octet writes its artifact directory
- THEN the receipt MUST preserve bounded logs and diagnostics.
- AND the receipt SHOULD name the reviewer action required before release promotion.

### Requirement: Optional Hegel property evidence rail
r[octet.dynamic_evidence.hegel_property_rail] Octet MUST expose Hegel RS property-test receipts as an opt-in external evidence rail and MUST NOT require Hegel for starter `cargo octet check`.

#### Scenario: Operator collects Hegel evidence explicitly
r[octet.dynamic_evidence.hegel_property_rail.explicit]
- GIVEN a workspace has produced Hegel property-test output for selected pure-core laws
- WHEN the operator runs the Hegel property evidence rail collection command
- THEN Octet MUST write scoped Hegel property receipts without changing starter check requirements.

#### Scenario: Starter check remains lightweight
r[octet.dynamic_evidence.hegel_property_rail.starter]
- GIVEN a consumer runs starter `cargo octet check`
- WHEN no Hegel rail is requested
- THEN Octet MUST NOT invoke Hegel, require Hegel artifacts, or bootstrap a Hegel backend.

### Requirement: Hegel property receipt identity
r[octet.dynamic_evidence.hegel_property_receipt] Hegel property receipts MUST bind property identity, source scope, linked requirement or policy IDs, Hegel runner identity, deterministic settings, generator identity, status, diagnostics, BLAKE3 artifact identities, and explicit non-claims.

#### Scenario: Complete receipt passes
r[octet.dynamic_evidence.hegel_property_receipt.complete]
- GIVEN a Hegel receipt contains property ID, checked scope, linked IDs, Hegel crate/backend identity, settings, case budget, phases, seed or derandomized mode, generator identity, status, artifact hashes, and non-claims
- WHEN Octet validates the receipt
- THEN validation MUST preserve those fields as deterministic generated-testing evidence metadata.

#### Scenario: Missing or stale identity fails closed
r[octet.dynamic_evidence.hegel_property_receipt.fail_closed]
- GIVEN a Hegel receipt lacks source scope, runner identity, settings, generator identity, artifact hashes, or required non-claims, or records a stale generator hash
- WHEN Octet validates the receipt
- THEN validation MUST fail with deterministic diagnostics naming the rejected field.

#### Scenario: Failing property preserves counterexample
r[octet.dynamic_evidence.hegel_property_receipt.counterexample]
- GIVEN a Hegel property run finds a minimal failing example
- WHEN Octet records the receipt
- THEN the receipt MUST preserve bounded failure diagnostics and a counterexample summary or content hash suitable for review and regression fixture creation.

### Requirement: Hegel evidence boundary
r[octet.dynamic_evidence.hegel_property_boundary] Hegel property receipts MUST remain scoped generated-testing evidence and MUST NOT be promoted to formal proof, exhaustive coverage, whole-program correctness, verifier soundness, or release eligibility.

#### Scenario: Passing Hegel run stays generated-testing evidence
r[octet.dynamic_evidence.hegel_property_boundary.generated_testing]
- GIVEN a Hegel property receipt reports a passing run
- WHEN Octet classifies the receipt
- THEN the evidence tier MUST represent bounded generated-input testing rather than formal proof.

#### Scenario: Promotion attempt fails closed
r[octet.dynamic_evidence.hegel_property_boundary.no_promotion]
- GIVEN a Hegel receipt attempts to claim formal proof, exhaustive correctness, semantic equivalence, requirement satisfaction, release eligibility, or verifier soundness
- WHEN Octet validates the receipt
- THEN validation MUST fail with a deterministic tier or overclaim diagnostic.

#### Scenario: Operator sees non-claims
r[octet.dynamic_evidence.hegel_property_boundary.docs]
- GIVEN an operator reads the Hegel rail guide or receipt summary
- WHEN the supported claim is described
- THEN the output MUST state that Hegel checks bounded generated inputs only and does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety.

### Requirement: Cargo-fuzz profiles are explicit and bounded

r[octet.dynamic_evidence.cargo_fuzz_profile]

Octet MUST define an opt-in cargo-fuzz profile with an exact engine, target, toolchain, command shape, execution budget, sanitizer posture, artifact limits, and non-claims.

#### Scenario: Valid fuzz profile loads

- GIVEN a reviewed profile names a supported engine, target selector, allowed argv shape, toolchain, budgets, limits, and non-claims
- WHEN Octet validates the profile
- THEN validation MUST produce one deterministic profile identity
- AND the identity MUST bind every behavior-affecting profile field

#### Scenario: Unbounded or arbitrary command fails

- GIVEN a profile omits a required limit or permits arbitrary command text, target selection, environment input, or output path
- WHEN profile validation runs
- THEN validation MUST fail before process execution
- AND Octet MUST NOT infer missing limits from ambient state

### Requirement: Cargo-fuzz receipts bind exact run and corpus evidence

r[octet.dynamic_evidence.cargo_fuzz_receipt]

Octet MUST emit a versioned receipt that binds the selected target, engine, compiler, sanitizer, argv, budgets, corpus, dictionary, result, crash artifacts, logs, and BLAKE3 identities.

#### Scenario: Clean bounded run records final corpus

- GIVEN a selected fuzz target completes its declared execution budget without a crash
- WHEN Octet records the result
- THEN the receipt MUST bind the initial and final canonical corpus roots
- AND it MUST distinguish a bounded clean observation from exhaustive input coverage

#### Scenario: Crash result records reproducible evidence

- GIVEN the fuzz engine discovers a crashing input and produces a minimized reproducer
- WHEN Octet records the result
- THEN the receipt MUST bind both crash artifacts and the exact replay command
- AND the failed status MUST remain reviewable without being rendered as a successful gate

#### Scenario: Stale or unsafe artifact fails

- GIVEN a corpus, dictionary, crash input, or reproducer has a stale hash, unsafe path, excessive size, or missing required role
- WHEN receipt validation runs
- THEN validation MUST fail with a deterministic artifact diagnostic
- AND artifact validation MUST NOT publish a passing receipt

### Requirement: Fuzz evidence remains bounded test evidence

r[octet.dynamic_evidence.cargo_fuzz_boundary]

Cargo-fuzz receipts MUST remain scoped generated-execution evidence and MUST NOT claim exhaustive coverage, panic freedom, memory safety, formal proof, or release eligibility.

#### Scenario: Passing run remains transcript evidence

- GIVEN a fuzz receipt reports no crash under its selected budget
- WHEN Octet classifies the receipt
- THEN the evidence tier MUST remain `transcript_assertion` or `manual_review_required`
- AND the summary MUST state the selected budget and non-claims

#### Scenario: Claim promotion fails

- GIVEN a fuzz receipt requests `formal_proof` or claims exhaustive input coverage, whole-program correctness, or verifier soundness
- WHEN Octet validates the receipt
- THEN validation MUST fail with a tier or overclaim diagnostic

### Requirement: Named dynamic profiles retain command isolation

r[octet.dynamic_evidence.named_profile_regression]

Octet MUST retain positive and negative consumer fixtures for each named Miri, Loom, and cargo-fuzz dynamic profile.

#### Scenario: Declared command is accepted

- GIVEN a fixture selects one named profile and its exact allowed command shape
- WHEN profile command resolution runs
- THEN Octet MUST construct the declared argv deterministically
- AND no other named profile command MUST be selected

#### Scenario: Cross-profile command is rejected

- GIVEN a fixture selects one profile but supplies another profile's command or artifact shape
- WHEN profile command resolution or receipt validation runs
- THEN Octet MUST fail with a deterministic profile-mismatch diagnostic

### Requirement: Cargo-fuzz verification covers positive and negative paths

r[octet.dynamic_evidence.cargo_fuzz_verification]

The implementation MUST include deterministic positive and negative coverage for profile validation, process classification, corpus identity, crash evidence, limits, command isolation, artifact validation, and default-path independence.

#### Scenario: Focused fixture matrix completes

- GIVEN the clean, crash, malformed, stale, unsafe, over-limit, tool-failure, and command-mismatch fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class

### Requirement: Hegel run mode uses a typed producer profile

r[octet.dynamic_evidence.hegel_profiled_run]

Hegel run mode MUST construct fixed argv from a validated Nickel producer profile and MUST reject arbitrary command text, executable selection, arguments, environment values, paths, or network authority.

#### Scenario: Named producer profile runs

- GIVEN a reviewed profile binds a producer package, binary target, property catalog, Hegel version, backend, settings, source scope, limits, retention policy, and non-claims
- WHEN the operator selects the profile in Hegel run mode
- THEN Octet MUST construct the declared argv deterministically
- AND the receipt MUST bind the profile identity and exact constructed argv

#### Scenario: Explicit command is supplied

- GIVEN an operator supplies `--command` or a profile contains free-form command text
- WHEN Hegel run preflight executes
- THEN Octet MUST reject the request before process execution

#### Scenario: Hegel is not selected

- GIVEN an operator runs starter checks or another evidence rail
- WHEN command resolution runs
- THEN Octet MUST NOT require or execute a Hegel producer, profile, adapter, or property test

### Requirement: Hegel runs use a dedicated output contract

r[octet.dynamic_evidence.hegel_output_contract]

A profiled Hegel producer MUST write one versioned summary artifact through an explicit capability-rooted output path, while stdout and stderr remain bounded diagnostic logs.

#### Scenario: Dedicated summary passes

- GIVEN the selected producer exits under its limits and writes a complete supported summary to the declared output path
- WHEN Octet evaluates the run
- THEN the pure core MUST validate the summary independently from process logs
- AND the receipt MUST preserve status, settings, property identity, source scope, generator identity, and any counterexample

#### Scenario: Mixed process output cannot become evidence

- GIVEN a producer writes test-harness text, JSON fragments, or unrelated diagnostics only to stdout or stderr
- WHEN Hegel run evaluation executes
- THEN Octet MUST report a missing or malformed summary artifact
- AND it MUST NOT parse the mixed logs as a passing Hegel receipt

#### Scenario: Compatible import remains bounded

- GIVEN an external `hegel-property-summary/v1` input satisfies the accepted import contract
- WHEN import mode runs
- THEN Octet MUST preserve its bounded generated-testing evidence classification
- AND Octet MUST NOT invent hardened run measurements that the input does not contain

### Requirement: Hegel run identities are measured

r[octet.dynamic_evidence.hegel_measured_identity]

Octet MUST remeasure and bind the selected profile, Cargo lock, producer package, source scope, generator inputs, summary artifact, bounded logs, and retained counterexample before publishing a profiled Hegel receipt.

#### Scenario: Declared and measured identities match

- GIVEN the producer summary declares facts that match Octet's measured profile, package, source, generator, output, and counterexample facts
- WHEN receipt validation runs
- THEN Octet MUST publish their BLAKE3 identities and parent links deterministically

#### Scenario: Self-declared identity is stale

- GIVEN a declared version, source hash, generator hash, profile hash, summary hash, or counterexample hash differs from measured input
- WHEN receipt validation runs
- THEN validation MUST fail with a deterministic identity diagnostic
- AND the failed artifact MUST NOT be presented as passing evidence

#### Scenario: Output path is unsafe

- GIVEN a producer output or counterexample uses parent traversal, an absolute path, symlink substitution, an undeclared file, or an over-limit artifact
- WHEN artifact collection runs
- THEN collection MUST fail closed without reading or replacing data outside the capability root

### Requirement: Hegel hardening has positive and negative verification

r[octet.dynamic_evidence.hegel_hardening_verification]

The implementation MUST include deterministic positive and negative coverage for profile authority, output contracts, result classification, counterexamples, replay, identities, limits, paths, compatibility, evidence tiers, and default independence.

#### Scenario: Focused hardening matrix completes

- GIVEN profiled-pass, counterexample, replay, import, command, schema, output, path, identity, limit, timeout, and overclaim fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
