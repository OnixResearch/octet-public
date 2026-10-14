## MODIFIED Requirements

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
