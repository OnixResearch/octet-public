## Phase 1: Implementation

- [x] I1 Add `openspec/templates/proposal.md` with the exact heading inventory and prompt lines from `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/specs/openspec-stage-authoring/spec.md`.
- [x] I2 Add `openspec/templates/design.md` with the exact heading inventory and deterministic-proof prompt labels from `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/specs/openspec-stage-authoring/spec.md`.
- [x] I3 Update `docs/contributor-guide.md` and `openspec/config.yaml` so both proposal-stage and design-stage guidance point at the two new template paths with the exact lines promised by `openspec-stage-authoring`.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Maintainer names exact proposal surfaces before drafting specs" with `python3 - <<'PY'
from pathlib import Path
lines = Path('openspec/templates/proposal.md').read_text().splitlines()
expected_h2 = [
    '## Why',
    '## What Changes',
    '## Non-Goals',
    '## Capabilities',
    '## Impact',
]
actual_h2 = [line for line in lines if line.startswith('## ')]
assert actual_h2 == expected_h2, actual_h2
expected_other = [
    '### New Capabilities',
    '### Modified Capabilities',
    '- **Files / artifacts**: <exact repo-relative paths touched by this change>',
    '- **Ordering / dependencies**: <what must land first or stay in lockstep>',
    '- **Verification / evidence**: <saved artifacts or helper/gate runs planned for review>',
]
for line in expected_other:
    assert line in lines, line
for line in actual_h2 + expected_other:
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/proposal-template-fields.txt`
- [x] V2 Verify scenario "Maintainer binds each proof artifact to path and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('openspec/templates/design.md').read_text().splitlines()
expected_h2 = [
    '## Context',
    '## Goals / Non-Goals',
    '## Decisions',
    '## Deterministic Proof Plan',
    '## Risks / Trade-offs',
]
actual_h2 = [line for line in lines if line.startswith('## ')]
assert actual_h2 == expected_h2, actual_h2
expected_other = [
    '- scenario or requirement:',
    '- artifact path:',
    '- command / helper:',
    '- ordering / prerequisites:',
    '- cleanup / postcondition:',
]
for line in expected_other:
    assert line in lines, line
for line in actual_h2 + expected_other:
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/design-template-fields.txt`
- [x] V3 Verify scenario "Contributor guide exposes both OpenSpec stage templates" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/contributor-guide.md').read_text().splitlines()
expected = [
    '[OpenSpec proposal template](../openspec/templates/proposal.md)',
    '[OpenSpec design template](../openspec/templates/design.md)',
]
for line in expected:
    assert line in lines, line
for line in expected:
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/openspec-stage-authoring-crosswalk.txt`
- [x] V4 Verify scenario "Config rules surface both templates to stage guidance" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/contributor-guide.md').read_text().splitlines()
config_text = Path('openspec/config.yaml').read_text()
expected_guide = [
    '[OpenSpec proposal template](../openspec/templates/proposal.md)',
    '[OpenSpec design template](../openspec/templates/design.md)',
]
for line in expected_guide:
    assert line in lines, line
assert 'openspec/templates/proposal.md' in config_text
assert 'openspec/templates/design.md' in config_text
for line in expected_guide:
    print(line)
print('proposal rule -> openspec/templates/proposal.md')
print('design rule -> openspec/templates/design.md')
PY`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/openspec-stage-authoring-crosswalk.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Maintainer names exact proposal surfaces before drafting specs` -> `V1` -> `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/proposal-template-fields.txt`
- `Scenario: Maintainer binds each proof artifact to path and order` -> `V2` -> `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/design-template-fields.txt`
- `Scenario: Contributor guide exposes both OpenSpec stage templates` -> `V3` -> `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/openspec-stage-authoring-crosswalk.txt`
- `Scenario: Config rules surface both templates to stage guidance` -> `V4` -> `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/openspec-stage-authoring-crosswalk.txt`
