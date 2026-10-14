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
