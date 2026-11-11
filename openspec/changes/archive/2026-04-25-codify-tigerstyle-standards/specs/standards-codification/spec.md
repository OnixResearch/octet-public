## ADDED Requirements

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
