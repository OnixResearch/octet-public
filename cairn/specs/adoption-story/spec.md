# Adoption Story Specification

## Purpose

Defines the `adoption-story` capability.

## Requirements

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

### Requirement: Public story card surface

r[octet.adoption_story.public_story_cards] Octet adoption documentation SHOULD provide a card-oriented public story surface that lets a first-time reader skim the checked-path promise, how Octet works, evidence-backed case studies, before/after examples, a terminal-first quickstart, and FAQ boundaries without reading archived changes.

#### Scenario: Story card inventory is visible

r[octet.adoption_story.public_story_cards.inventory]

- GIVEN a first-time reader opens the canonical adoption-story surface
- WHEN they scan second-level headings or card labels
- THEN they SHOULD see sections for the checked path, how it works, Built with Octet evidence, before/after examples, quickstart, and FAQ.
- AND the sections SHOULD point to concrete commands, artifacts, lints, policy checks, proof receipts, or existing reference docs.

#### Scenario: How-it-works cards name Octet surfaces

r[octet.adoption_story.public_story_cards.how_it_works]

- GIVEN the adoption story includes a how-it-works section
- WHEN a reader inspects the cards
- THEN the cards SHOULD name static lint edge, Nickel policy layer, evidence chain, optional scoped proof lane, artifact verification, and controlled deployment as distinct surfaces.
- AND each card SHOULD avoid upgrading validation evidence into formal proof.

### Requirement: Terminal-first quickstart path

r[octet.adoption_story.quickstart_path] Octet adoption documentation SHOULD include a short terminal-first quickstart that shows the command sequence from first check to saved artifacts and artifact verification, including expected output shape or artifact names.

#### Scenario: Quickstart reaches reviewable artifacts

r[octet.adoption_story.quickstart_path.artifacts]

- GIVEN a reader follows the quickstart
- WHEN they run the documented commands in a configured Rust workspace
- THEN the quickstart SHOULD lead them through `cargo octet check` with an artifact directory and an artifact verification command.
- AND the quickstart SHOULD name expected reviewable outputs such as `status.json`, `summary.txt`, SARIF or JSON findings, and `provenance.jsonl` when those outputs are selected.

### Requirement: Before and after examples

r[octet.adoption_story.before_after_examples] Octet adoption documentation SHOULD include before/after examples for high-confidence lint families that show ordinary Rust failure shapes and Octet-aligned remediation paths.

#### Scenario: Example names lint, failure shape, and remediation

r[octet.adoption_story.before_after_examples.fields]

- GIVEN a before/after example appears in the adoption story
- WHEN a reviewer inspects it
- THEN the example SHOULD name the relevant Octet lint or policy surface.
- AND it SHOULD name the deterministic failure shape, missing guard, resource boundary, or remediation path.
- AND it SHOULD show a bounded remediation such as an explicit guard, checked conversion, bounded queue, shell/core split, safe wrapper, or explicit error propagation.

#### Scenario: Example avoids formal-counterexample wording

r[octet.adoption_story.before_after_examples.no_counterexample_overclaim]

- GIVEN a before/after example is backed only by lint or policy analysis
- WHEN the text explains the finding
- THEN it MUST NOT call the example an SMT model, formal counterexample, mathematical proof, or solver result.

### Requirement: FAQ boundaries

r[octet.adoption_story.faq_boundaries] Octet adoption documentation SHOULD include a compact FAQ that answers why not only Clippy, what Octet proves, what Octet does not prove, why Dylint or nightly is used, what data leaves the repository, how starter-profile noise is handled, and when `cargo octet prove` is appropriate.

#### Scenario: FAQ covers comparison and operations topics

r[octet.adoption_story.faq_boundaries.topics]

- GIVEN the adoption story includes an FAQ
- WHEN a reader scans the FAQ question list
- THEN it SHOULD include questions about Clippy comparison, proof scope, unsupported proof claims, Dylint or nightly tradeoffs, source/artifact locality, starter-profile rollout noise, and `cargo octet prove` usage.

#### Scenario: FAQ pairs claim with boundary

r[octet.adoption_story.faq_boundaries.claim_boundary]

- GIVEN an FAQ answer describes an Octet strength
- WHEN that strength involves proof, verification, safety, or deployment locality
- THEN the answer MUST pair the supported surface with its explicit boundary or non-claim.

### Requirement: Salt inspiration boundaries

r[octet.adoption_story.salt_inspiration_boundaries] Salt-inspired Octet presentation MUST preserve Octet's Rust-compatible checked-path scope and MUST NOT claim Salt-style language syntax, mathematical certainty, C performance parity, a verified runtime, certification, or whole-program proof.

#### Scenario: Salt-style overclaim fails validation

r[octet.adoption_story.salt_inspiration_boundaries.overclaim_check]

- GIVEN documentation validation scans adoption-story and landing-page copy
- WHEN it encounters a claim that Octet has built-in `requires` or `ensures` syntax, guarantees mathematical certainty, matches C performance, certifies systems, provides a verified runtime, or proves whole-program correctness
- THEN validation MUST fail unless the claim is explicitly negated or scoped to a separate accepted proof artifact.

#### Scenario: External inspiration is not a dependency

r[octet.adoption_story.salt_inspiration_boundaries.no_external_runtime_dependency]

- GIVEN the story surface was inspired by an external project page
- WHEN documentation validation runs
- THEN validation MUST use checked-in Octet files and fixtures only.
- AND it MUST NOT fetch external pages or treat external claims as Octet evidence.

### Requirement: First checked Rust patch walkthrough

r[octet.adoption_story.first_checked_patch_walkthrough] Octet adoption documentation SHOULD include a self-contained first checked Rust patch walkthrough that demonstrates an existing Rust source finding, bounded remediation, local evidence artifact production, artifact verification, and explicit claim boundaries.

#### Scenario: Walkthrough triggers a deterministic finding

r[octet.adoption_story.first_checked_patch_walkthrough.finding]

- GIVEN the adoption story links a first checked Rust patch walkthrough
- WHEN a reader inspects the walkthrough's initial example
- THEN the walkthrough SHOULD show a small before-code sample that triggers one named existing Octet lint or policy surface.
- AND it SHOULD name the deterministic failure shape, missing guard, resource boundary, ignored result, or other remediation reason behind the finding.
- AND it MUST NOT describe the finding as a solver model, formal counterexample, mathematical proof, or verifier result unless backed by a scoped proof receipt.

#### Scenario: Walkthrough fixes the patch and verifies artifacts

r[octet.adoption_story.first_checked_patch_walkthrough.artifacts]

- GIVEN a reader follows the walkthrough commands in a configured Rust workspace
- WHEN they apply the documented remediation
- THEN the walkthrough SHOULD show the fixed-code shape and rerun `cargo octet check --artifact-dir artifacts/octet` or an equivalent local artifact-producing command.
- AND it SHOULD end with `cargo octet artifact verify --artifact-dir artifacts/octet` or an equivalent artifact-verification command.
- AND it SHOULD name expected reviewable local outputs such as `status.json`, `summary.txt`, findings sidecars, SARIF or JSONL when selected, and `provenance.jsonl`.

#### Scenario: Walkthrough keeps onboarding and proof boundaries explicit

r[octet.adoption_story.first_checked_patch_walkthrough.boundaries]

- GIVEN the walkthrough explains the value of the checked path
- WHEN the text describes evidence, verification, proof, or release readiness
- THEN it MUST state that the walkthrough demonstrates static lint or policy evidence and artifact verification for the selected scope.
- AND it MUST NOT claim Rust replacement behavior, Salt-style `requires` or `ensures` syntax, mathematical certainty, whole-program correctness, certification, C performance parity, or a verified runtime.
- AND it SHOULD link to the full consumer guide or adoption story for production setup and broader rollout guidance.
