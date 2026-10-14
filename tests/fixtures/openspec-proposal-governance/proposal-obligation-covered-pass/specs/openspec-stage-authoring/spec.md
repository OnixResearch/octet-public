## MODIFIED Requirements

### Requirement: Proposal Template Captures Scope, Artifacts, And Ordering

The system MUST require proposal authors to state exact proposal obligations before later-stage authoring.

#### Scenario: Proposal author records exact scope and downstream validation boundaries

- GIVEN a proposal affects the concrete fixture pair `proposal-obligation-missing-blocked` and `proposal-obligation-covered-pass`
- WHEN the proposal is prepared for gate review
- THEN it names exact affected artifacts including `openspec/templates/proposal.md`, `openspec/config.yaml`, `openspec/specs/openspec-stage-authoring/spec.md`, and `tests/fixtures/openspec-proposal-governance/**`
- AND it names downstream validation boundaries that exclude Aspen, Valence object corpus fixtures, and historical archive regeneration
- AND it names saved evidence outputs including `PASS expected passing fixture: proposal-obligation-covered-pass`

#### Scenario: Proposal governance fixtures prove blocked and covered obligations

- GIVEN the proposal-governance fixture pair `proposal-obligation-missing-blocked` and `proposal-obligation-covered-pass`
- WHEN the runner executes the missing fixture
- THEN the gate blocks the missing fixture because exact proposal artifacts, concrete scope, downstream validation boundaries, and saved evidence expectations are omitted
- AND the missing-fixture transcript includes the target proposal omission finding
- WHEN the runner executes the covered fixture
- THEN the proposal gate passes because exact obligations are present
- AND the covered-fixture transcript includes `PASS expected passing fixture: proposal-obligation-covered-pass`

#### Scenario: Static helper outputs prove proposal obligation fields

- GIVEN the proposal obligation helper checks the template, config, and fixtures
- WHEN the helper runs `template`, `config`, and `fixtures` modes
- THEN the saved outputs include `PASS proposal template includes obligation fields`
- AND the saved outputs include `PASS config includes proposal-stage obligation guidance`
- AND the saved outputs include `PASS proposal-obligation fixtures encode missing and covered obligations`
