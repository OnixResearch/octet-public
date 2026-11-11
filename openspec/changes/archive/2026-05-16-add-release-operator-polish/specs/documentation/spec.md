## MODIFIED Requirements

### Requirement: README Landing Page

The repo MUST keep `README.md` as a newcomer landing page instead of the only full manual.

That landing page MUST, near the top of the file:

- state what tigerstyle is
- show the first-class consumer run path with `cargo tigerstyle check` and the matching `nix run ...#cargo-tigerstyle -- check` form
- link to deeper docs by audience instead of embedding every rollout detail in the README itself
- identify the release-critical operator surfaces for `fingerprint`, `migrate`, effect-summary output, lint-cache behavior, and executable transcript examples
- point operators to one deterministic smoke transcript or docs-example gate that covers those release-critical surfaces together

#### Scenario: New user chooses an install/run path [r[readme-landing-page.scenario.new-user-chooses-an-install-run-path]]

- GIVEN a new user opens `README.md`
- WHEN they read the first setup section
- THEN they can choose between the installed `cargo tigerstyle check` path and the `nix run ...#cargo-tigerstyle -- check` path without reading archive specs

#### Scenario: Operator finds release-critical surfaces [r[readme-landing-page.scenario.operator-finds-release-critical-surfaces]]

- GIVEN an operator is preparing a release smoke run
- WHEN they read `README.md` and command help
- THEN they can find the `fingerprint`, `migrate`, effect-summary, lint-cache, and executable transcript surfaces from the release-facing docs/help path

### Requirement: Executable Documentation Transcripts [r[executable-documentation-transcripts]]

The system MUST support executable documentation transcripts for Slotcar examples. A transcript MUST be a Markdown document containing Slotcar command blocks, fixture setup references, and expected-output assertions. Running a transcript MUST produce deterministic output artifacts suitable for tests and OpenSpec evidence.

#### Scenario: Transcript executes command blocks [r[executable-documentation-transcripts.scenario.transcript-executes-command-blocks]]

- GIVEN a docs example contains a `slotcar-check` command block and fixture reference
- WHEN `cargo slotcar transcript <path>` runs
- THEN the command executes in an isolated fixture workspace and records exit status plus bounded output

#### Scenario: Expected output assertion fails closed [r[executable-documentation-transcripts.scenario.expected-output-assertion-fails-closed]]

- GIVEN a transcript expected-output block names a lint that no longer appears
- WHEN the transcript runner evaluates the assertion
- THEN the command exits non-zero and reports the failed assertion location

#### Scenario: Transcript artifact is content addressed [r[executable-documentation-transcripts.scenario.transcript-artifact-is-content-addressed]]

- GIVEN the same transcript, fixture, command, and tool revision are run twice
- WHEN the transcript runner writes its artifact header
- THEN the transcript hash fields are identical across runs

#### Scenario: Release smoke transcript covers operator surfaces [r[executable-documentation-transcripts.scenario.release-smoke-transcript-covers-operator-surfaces]]

- GIVEN an operator wants one deterministic release-readiness smoke artifact
- WHEN they run the documented smoke transcript or docs-example gate
- THEN the saved transcript output names `fingerprint`, `migrate`, effect-summary, lint-cache, and executable transcript behavior in one bounded artifact
