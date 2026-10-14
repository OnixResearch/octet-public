## ADDED Requirements

### Requirement: Rust-compatible adoption positioning

r[octet.adoption_story.positioning] Octet documentation MUST present Octet as a Rust-compatible, evidence-backed checked path for systems code before release, not as a new programming language or Rust replacement.

#### Scenario: Reader sees the checked-path promise

r[octet.adoption_story.positioning.promise]

- GIVEN a reader opens the primary Octet adoption or README surface
- WHEN they read the introductory positioning
- THEN the documentation MUST state that Octet checks Rust systems code before release through runner, lint, standards, evidence, and artifact-verification surfaces.
- AND the documentation MUST NOT describe Octet as a new programming language or replacement for Rust.

#### Scenario: Reader can follow the checked path

r[octet.adoption_story.positioning.path]

- GIVEN the adoption story describes how Octet is used
- WHEN a reader follows the path from source to release evidence
- THEN the documentation SHOULD name `cargo octet check`, checked-in config or policy, evidence artifacts, CI or release gates, and artifact verification as review surfaces.

### Requirement: Evidence-backed case studies

r[octet.adoption_story.case_studies] Octet documentation SHOULD provide a repeatable "Built with Octet" case-study format, and each published case study MUST name its checked scope, command or gate, configuration source, evidence artifact or archive path, outcome summary, and explicit scope boundary.

#### Scenario: Case study is reviewable

r[octet.adoption_story.case_studies.reviewable]

- GIVEN a "Built with Octet" case study appears in public documentation
- WHEN a reviewer inspects the entry
- THEN the entry MUST name the consumer or archived evidence source.
- AND the entry MUST name the checked package or source scope.
- AND the entry MUST name the Octet command, CI gate, or release gate that produced the evidence.
- AND the entry MUST name the relevant configuration source.
- AND the entry MUST name the evidence artifact, transcript, or archive path.
- AND the entry MUST state the outcome summary and scope boundary.

#### Scenario: Aspen phase-one evidence uses canonical archive

r[octet.adoption_story.case_studies.aspen_archive]

- GIVEN an Octet case study uses Aspen phase-1 starter-rollout evidence
- WHEN the documentation names the evidence source
- THEN it MUST cite `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` as the canonical reference.
- AND it MUST NOT rely only on a mutable local Aspen worktree path.

### Requirement: Methodology before public metrics

r[octet.adoption_story.methodology] Public Octet performance, cache, CI-latency, suppression-rate, false-positive, or rollout-cost claims SHOULD include the command, input scope, environment, raw artifact or transcript path, and interpretation boundary used to produce the claim.

#### Scenario: Published metric includes method

r[octet.adoption_story.methodology.metric_fields]

- GIVEN Octet documentation publishes a performance, cache, CI-latency, suppression-rate, false-positive, or rollout-cost metric
- WHEN a reviewer inspects that metric
- THEN the documentation SHOULD name the command or gate that produced it.
- AND it SHOULD name the input scope and configuration.
- AND it SHOULD name the environment or platform when environment affects the result.
- AND it SHOULD name the raw artifact, transcript, or reproducible evidence path.
- AND it SHOULD state the interpretation boundary for the metric.

#### Scenario: Metric without method is held back

r[octet.adoption_story.methodology.no_orphan_metric]

- GIVEN an adoption-story draft contains a public metric without methodology fields
- WHEN documentation validation runs
- THEN validation SHOULD fail or mark the metric as not ready for publication.

### Requirement: Adoption story preserves claim boundaries

r[octet.adoption_story.claim_boundaries] Octet adoption documentation MUST NOT claim mathematical certainty, whole-program correctness, certification, a verified runtime, or Rust replacement behavior unless a separate accepted requirement and evidence artifact explicitly supports that claim.

#### Scenario: Unsupported overclaim fails validation

r[octet.adoption_story.claim_boundaries.overclaim_check]

- GIVEN documentation validation scans adoption-story docs
- WHEN it encounters an unsupported claim that Octet guarantees mathematical certainty, proves whole-program correctness, certifies a system, provides a verified runtime, or replaces Rust
- THEN validation MUST fail with a diagnostic naming the unsupported claim.

#### Scenario: External comparison remains bounded

r[octet.adoption_story.claim_boundaries.external_comparison]

- GIVEN the adoption story compares Octet with a new-language or verifier-first tool
- WHEN the comparison describes Octet's strengths
- THEN the comparison MUST keep Octet's claim scoped to Rust-compatible static checks, policy checks, optional scoped proof receipts, provenance, and artifact verification.
- AND it MUST NOT imply unsupported equivalence with language-level proof systems.
