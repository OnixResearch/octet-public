# Open Issue Resolution Specification Delta

## Purpose

Define the execution-coverage, requirement-contract, freshness, replay,
authentication, scanner, benchmark, and release-cohort behavior added to close
the nine reviewed open issues. Every capability keeps its bounded non-claims.

## ADDED Requirements

### Requirement: Policy execution coverage is derived from checker outcomes

r[octet.open_issue_resolution.policy_execution]

Octet MUST report one execution outcome per registry policy derived from actual
checker applicability and diagnostics. A selected local policy with no
implemented checker MUST be `unsupported`; an externally owned or human-review
policy MUST be `not_run` with explicit ownership; a checker failure MUST be
`tool_error`.

#### Scenario: Unsupported local policies block strict coverage
- GIVEN a selected hard local policy has no implemented checker
- WHEN the standards report is rendered
- THEN the policy MUST report `unsupported`
- AND strict policy coverage MUST NOT pass.

#### Scenario: External and human checks stay separate
- GIVEN a selected required policy is owned by an external tool or human review
- WHEN the local standards run completes
- THEN the policy MUST report `not_run` and MUST NOT be represented as a completed local check
- AND strict policy coverage MUST NOT pass while it remains `not_run`.

#### Scenario: Passing and failing local checks retain scope
- GIVEN a local checker ran on an applicable source scope
- WHEN it emits no diagnostic for the policy
- THEN the policy MUST report `passed` with the checked scope
- AND a diagnostic for the policy MUST report `failed` with the linked diagnostic identities.

### Requirement: Negative controls require attributable verifier rejection

r[octet.open_issue_resolution.negative_control]

Octet MUST accept a proof negative control only when retained verifier evidence
shows a passing baseline and a mutant rejection attributed to the expected
obligation, with both runs bound to the same specification, scope, provider,
toolchain, configuration, and mutation identity.

#### Scenario: Unattributed failure is inconclusive
- GIVEN a mutant card reports `failed` without typed verifier evidence
- WHEN the negative control is recorded or validated
- THEN the result MUST be `inconclusive` and MUST NOT be `property_demonstrated`.

#### Scenario: Retained artifacts are recomputed
- GIVEN a recorded v2 negative control
- WHEN proof-ref validation runs after a source, deck, mutation, obligation, or specification change
- THEN validation MUST fail with the changed identity.

### Requirement: Reviewed requirement contracts are executable

r[octet.open_issue_resolution.requirement_contract]

Octet MUST evaluate a versioned, Nickel-authored requirement contract into a
Rust-owned decision that binds reviewed specification, source, clause,
assumption, exclusion, and evidence identities and can be recomputed without an
AI model or hosted provider.

#### Scenario: Missing clause evidence fails
- GIVEN every other clause has valid evidence and one required clause has none
- WHEN the contract is evaluated
- THEN the aggregate result MUST fail
- AND the missing clause MUST be reported with its required disposition.

#### Scenario: Empirical evidence cannot satisfy formal proof
- GIVEN a formal-proof clause has only empirical evidence
- WHEN the contract is evaluated
- THEN the clause MUST fail and MUST NOT be promoted to formal proof.

#### Scenario: Changed assumptions and decisions are detected
- GIVEN a decision was recorded against an earlier assumption identity or contract
- WHEN the decision is verified after the assumption, scope, or evidence changes
- THEN verification MUST fail with the changed identity or stale decision bytes.

### Requirement: Required-evidence freshness consumes the reviewed contract

r[octet.open_issue_resolution.freshness]

Octet MUST provide a read-only freshness operation that consumes the reviewed
requirement contract as its expected-obligation inventory and compares it with
an evidence codebase index using validated direct bindings.

#### Scenario: Fresh and stale receipts are distinguished
- GIVEN a current receipt satisfies the declared bindings
- WHEN the same claim has an older receipt with changed source or policy identities
- THEN the current receipt MUST report `current`
- AND the older receipt MUST report `stale_binding` with old and new identities.

#### Scenario: Incomplete inventory is not missing
- GIVEN a required clause has no matching evidence and the index is partial
- WHEN the freshness query runs
- THEN the clause MUST report `incomplete`, not `missing`
- AND shared labels MUST NOT be treated as dependency.

### Requirement: Executed replay launches admitted profiles

r[octet.open_issue_resolution.executed_replay]

Octet MUST preserve supplied-observation capsule replay as a distinct evidence
mode and MUST add an opt-in executed mode that launches only a reviewed closed
profile with fixed argv, allowlisted environment, bounded output, and a retained
execution receipt linked to the capsule.

#### Scenario: Fabricated observation cannot satisfy executed replay
- GIVEN an executed replay request and a supplied observation file
- WHEN the job has not been launched
- THEN the result MUST NOT report successful replay.

#### Scenario: Drift and expected failure are precise
- GIVEN an admitted profile ran successfully
- WHEN source, configuration, toolchain, argv, output, or exit identity changes
- THEN replay MUST report the precise drift
- AND expected-failure replay MUST require the named failure contract, not any nonzero exit.

### Requirement: Admission authentication is optional and externally anchored

r[octet.open_issue_resolution.admission_auth]

Octet MUST provide an opt-in trusted-producer admission path using the pinned
artifact-auth crates, with consumer-owned policy and evidence outside the
candidate bundle, and MUST keep unsigned local integrity verification as a
separate mode.

#### Scenario: Candidate trust roots are rejected
- GIVEN a candidate bundle supplies its own policy, keys, or `verified: true` observation
- WHEN authenticated admission runs against the consumer-owned policy
- THEN the candidate inputs MUST be rejected or ignored
- AND coherent whole-bundle replacement MUST fail against the original policy.

#### Scenario: Decisions are recomputable and separated
- GIVEN a valid signature from an authorized producer
- WHEN authenticated admission completes
- THEN integrity, signature authenticity, authorization, currentness, checker, and overall outcomes MUST be reported separately
- AND the decision receipt identity MUST be recomputable from retained inputs.

### Requirement: Local scanner execution is bounded and truthful

r[octet.open_issue_resolution.local_scanners]

Octet MUST provide opt-in Semgrep and CodeQL producer profiles above the SARIF
importer with pinned executable, rules/query packs, source snapshot, fixed argv,
resource limits, process outcome, and raw report identities; imported and
executed evidence MUST be labeled separately.

#### Scenario: Missing tool is not clean
- GIVEN the selected producer executable is absent
- WHEN producer execution runs
- THEN the outcome MUST be `unsupported` or `tool_error`
- AND an empty report MUST NOT establish a clean result.

#### Scenario: CodeQL database drift fails
- GIVEN a CodeQL database bound to another source snapshot, build configuration, executable, or query packs
- WHEN analysis runs
- THEN the database MUST be rejected as stale or mismatched.

### Requirement: Cross-tool experiment is separate from performance comparison

r[octet.open_issue_resolution.cross_tool_benchmark]

Octet MUST keep the same-command benchmark comparator unchanged and MUST add a
separate experiment contract with predeclared tasks, applicability, frozen
adjudicated outcomes, retained observations, comparable-subset metrics, adoption
costs, and negative controls.

#### Scenario: Unsupported scope is neither clean nor a false negative
- GIVEN a tool does not support a declared task or scope
- WHEN experiment metrics are derived
- THEN the case MUST remain unsupported and MUST be excluded from comparable precision and recall.

#### Scenario: Artifact verification recomputes derived fields
- GIVEN a retained experiment artifact
- WHEN verification runs
- THEN derived comparison fields MUST be recomputed from retained observations and controls
- AND tampering MUST fail verification.

### Requirement: Public release cohorts are qualified or explicitly blocked

r[octet.open_issue_resolution.release_cohort]

Octet MUST bind one supported public channel to exact source, export, dependency,
toolchain, package, notice, capability, and acceptance identities, and MUST fail
closed on source/inventory mismatch, omitted required notices, unresolved
imported prose represented as cleared, private inputs hidden by a warm cache,
copied historical receipts, or unsupported profiles represented as verified.

#### Scenario: Public inventory and notices are checked
- GIVEN a reviewed public revision and its own lint inventory and toolchain
- WHEN cohort qualification runs
- THEN the manifest MUST match that inventory and MUST list the required notice paths
- AND the current cohort MUST remain blocked while the public tree omits them.

#### Scenario: Development receipts are not public evidence
- GIVEN a development receipt or a receipt from a different source revision
- WHEN it is presented as acceptance evidence for the public cohort
- THEN validation MUST reject it.
