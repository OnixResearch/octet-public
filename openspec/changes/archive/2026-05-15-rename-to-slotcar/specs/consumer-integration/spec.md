## ADDED Requirements

### Requirement: Slotcar Canonical CLI Surface

The system MUST expose `cargo slotcar` / `cargo-slotcar` as the canonical downstream command surface for check runs, structured output, summaries, SARIF tool metadata, Nix package/app outputs, and consumer templates. If the old `cargo tigerstyle` / `cargo-tigerstyle` path remains available during transition, it MUST be documented as a compatibility alias and MUST delegate to the same implementation and output schema as `slotcar`.

The deterministic proof artifact `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/cli-surface.txt` MUST show the canonical binary/package surfaces include `slotcar` and that any supported compatibility alias exits successfully with equivalent behavior for a minimal fixture.

#### Scenario: Cargo subcommand uses slotcar

- GIVEN a consumer workspace with the repo-provided tool installed from this checkout
- WHEN the consumer runs `cargo slotcar check`
- THEN the command executes the same lint pipeline as the prior first-class cargo wrapper and reports the tool identity as `slotcar`

#### Scenario: Structured outputs name slotcar

- GIVEN a check run emits human summary, JSON, or SARIF output
- WHEN a consumer inspects tool-name fields, delimiters, or headings in the output
- THEN product/tool identity fields use `slotcar` instead of `tigerstyle`, except for explicitly documented compatibility or lint-namespace fields

#### Scenario: Compatibility alias is explicit

- GIVEN the implementation keeps `cargo tigerstyle` available during the rename transition
- WHEN a consumer invokes that old command path
- THEN it is documented and tested as a compatibility alias to the canonical `cargo slotcar` behavior rather than a second independent tool surface
