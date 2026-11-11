## Phase 1: Implementation

- [x] I1 Add `docs/assertion-review.md` with the exact guide headings from `openspec/changes/archive/2026-04-18-add-assertion-review-kit/specs/safety/spec.md`, reviewer prompts under every required heading, explicit guidance on assertion targets, paired checks, positive/negative space, split checks, compile-time assertions, and invalid-path testing, and place the why-the-checklist-matters rationale plus fail-fast framing for programmer errors and the rule that assertion philosophy is not only about raw density under `## Reviewer prompts and failure handling`.
- [x] I2 Add `templates/assertion-checklist.md` with the exact template headings from `openspec/changes/archive/2026-04-18-add-assertion-review-kit/specs/safety/spec.md`, make every section a fill-in field or prompt rather than free-form prose, and include prompts for contract assertions, paired sites, positive/negative space cases, compile-time assertions, invalid-path tests, and open risks.
- [x] I3 Add exact Markdown links `[Assertion review guide](docs/assertion-review.md)` and `[Assertion checklist template](templates/assertion-checklist.md)` as standalone lines at the top of `README.md`'s existing `## Documentation` section, and add exact Markdown links `[Assertion review guide](assertion-review.md)`, `[Assertion checklist template](../templates/assertion-checklist.md)`, and `[Safety spec contract](../openspec/specs/safety/spec.md)` as standalone lines near the top of `docs/contributor-guide.md` after the intro paragraphs and before `## Source-of-truth map`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/assertion-review.md templates/assertion-checklist.md`; input=`draft guide plus copyable template`; asserts=`Tiger Style maintainer reviews the full guide and full template, confirms each required topic sits under its designated heading, confirms pair assertions and invalid-path testing stay explicit, confirms `## Reviewer prompts and failure handling` specifically carries the why-the-checklist-matters rationale plus fail-fast framing and the rule that assertion philosophy is not only about raw density, records reviewer role plus reviewed artifact paths naming docs/assertion-review.md and templates/assertion-checklist.md, and either (a) writes the exact labeled lines with pass answers plus verdict: pass, or (b) writes the same exact labeled lines with one or more fail answers plus verdict: fail and sends the change back to I1-I3 before any V-task may pass`; evidence=`openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Guide keeps required headings and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/assertion-review.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Assertion targets: arguments, returns, invariants',
    '## Pair assertions across boundaries',
    '## Positive and negative space',
    '## Implication assertions and split checks',
    '## Compile-time assertions',
    '## Invalid-path testing',
    '## Reviewer prompts and failure handling',
]
assert headings == expected, f'unexpected guide headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- [x] V3 Verify scenario "Template keeps required headings and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('templates/assertion-checklist.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Function, boundary, or invariant under review',
    '## Assertions on inputs, outputs, and invariants',
    '## Paired assertion sites',
    '## Positive and negative space cases',
    '## Implications and split assertions',
    '## Compile-time assertions',
    '## Invalid-path tests',
    '## Open risks and follow-ups',
]
assert headings == expected, f'unexpected template headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-template-headings.txt`
- [x] V4 Verify scenario "Reader finds the assertion guide from README" with `python3 - <<'PY'
from pathlib import Path
for path in ['docs/assertion-review.md', 'templates/assertion-checklist.md']:
    assert Path(path).is_file(), path
    print(f'exists:{path}')
readme = Path('README.md').read_text().splitlines()
for line in [
    '[Assertion review guide](docs/assertion-review.md)',
    '[Assertion checklist template](templates/assertion-checklist.md)',
]:
    assert line in readme, line
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/readme-assertion-links.txt`
- [x] V5 Verify scenario "Contributor sees assertion doc touchpoints" with `python3 - <<'PY'
from pathlib import Path
for path in ['docs/assertion-review.md', 'templates/assertion-checklist.md', 'openspec/specs/safety/spec.md']:
    assert Path(path).is_file(), path
    print(f'exists:{path}')
lines = Path('docs/contributor-guide.md').read_text().splitlines()
for line in [
    '[Assertion review guide](assertion-review.md)',
    '[Assertion checklist template](../templates/assertion-checklist.md)',
    '[Safety spec contract](../openspec/specs/safety/spec.md)',
]:
    assert line in lines, line
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- [x] V6 Verify scenario "Saved review record keeps required labels" only after `H1` has saved `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`, with `python3 - <<'PY'
from pathlib import Path
import re
path = Path('openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt')
text = path.read_text()
expected_lines = [
    'reviewer role: Tiger Style maintainer',
    'reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md',
    'question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading',
    'question 2: pass - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings',
    'question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose',
    'question 4: pass - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density',
    'verdict: pass',
]
for line in expected_lines:
    matches = re.findall(rf'^{re.escape(line)}$', text, flags=re.MULTILINE)
    assert len(matches) == 1, f'{line} count={len(matches)}'
    print(f'{line}:1')
PY`; save output at `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review-labels.txt`

## Change Status
- Status: completed
- Open tasks: none
## Verification Coverage

- `Scenario: Guide keeps required headings and order` -> `V1` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- `Scenario: Reviewing a boundary with caller and callee checks` -> `H1` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
- `Scenario: Template keeps required headings and order` -> `V3` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-template-headings.txt`
- `Scenario: Planning invalid-path tests` -> `H1` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
- `Scenario: Saved guide heading proof is reviewable` -> `V1` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- `Scenario: Saved template heading proof is reviewable` -> `V3` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-template-headings.txt`
- `Scenario: Saved README link proof is reviewable` -> `V4` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/readme-assertion-links.txt`
- `Scenario: Saved contributor link proof is reviewable` -> `V5` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- `Scenario: Saved review-label proof is reviewable` -> `V6` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review-labels.txt`
- `Scenario: Reader finds the assertion guide from README` -> `V4` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/readme-assertion-links.txt`
- `Scenario: Contributor sees assertion doc touchpoints` -> `V5` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- `Scenario: Reviewer signs off assertion coverage` -> `H1` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
- `Scenario: Saved review record keeps required labels` -> `V6` -> `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review-labels.txt`
