## Why

Proposal omissions keep recurring when authors do not name exact evidence, artifacts, concrete scope, downstream boundaries, or non-goals before drafting later artifacts.

## What Changes

- Add copyable proposal guidance for exact proposal obligations.
- Add deterministic proposal-governance fixtures that prove a missing proposal obligation is blocked and a covered proposal passes.

## Non-Goals

- Do not mutate archived evidence outside `openspec/changes/archive/<date>-proposal-obligation-covered-pass/` if this fixture is ever archived.
- Do not change runtime lint behavior, cargo-octet behavior, or object corpus fixtures.
- Do not broaden into tasks-stage human/oracle checkpoint hardening.

## Capabilities

### Modified Capabilities

- `openspec-stage-authoring`: proposal guidance requires exact evidence, artifact, concrete scope, downstream-boundary, and non-goal obligations.

## Impact

- **Files / artifacts**: `openspec/templates/proposal.md`, `openspec/config.yaml`, `openspec/specs/openspec-stage-authoring/spec.md`, `tests/fixtures/openspec-proposal-governance/proposal-obligation-covered-pass/**`.
- **Concrete scope**: only the proposal-stage governance fixture pair `proposal-obligation-missing-blocked` and `proposal-obligation-covered-pass`; no design/spec/tasks governance fixtures are changed by this fixture.
- **Downstream validation boundaries**: fixture gate evidence remains local to `tests/fixtures/openspec-proposal-governance/`; no Aspen, Valence object corpus, or historical archive evidence is regenerated.
- **Ordering / dependencies**: proposal template/config/spec changes must land with the fixture pair and runner.
- **Verification / evidence**: `tests/fixtures/openspec-proposal-governance/run-proposal-fixture-gate.sh proposal-obligation-covered-pass` must print `PASS expected passing fixture: proposal-obligation-covered-pass`; static helper checks must print `PASS proposal template includes obligation fields`, `PASS config includes proposal-stage obligation guidance`, and `PASS proposal-obligation fixtures encode missing and covered obligations`.
