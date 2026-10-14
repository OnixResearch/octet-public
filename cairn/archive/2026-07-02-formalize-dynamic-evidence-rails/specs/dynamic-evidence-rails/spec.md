## ADDED Requirements

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
