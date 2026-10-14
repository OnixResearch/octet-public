## MODIFIED Requirements

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
