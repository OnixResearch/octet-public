## ADDED Requirements

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

