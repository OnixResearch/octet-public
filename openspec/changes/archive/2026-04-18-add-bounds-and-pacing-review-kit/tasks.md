## Phase 1: Implementation

- [x] I1 Add `docs/bounds-and-pacing-review.md` with the exact guide headings from `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/specs/safety/spec.md`, reviewer prompts under every required heading, explanation of why the checklist matters, allocation lifetime guidance, startup sizing guidance, queue/backpressure guidance, external-event pacing guidance, batch/work budgeting guidance, overload behavior guidance, and explicit content for audited exceptions plus review cadence under `## Reviewer prompts and escape hatches`.
- [x] I2 Add `templates/bounds-and-pacing-checklist.md` with the exact template headings from `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/specs/safety/spec.md`, make every section a fill-in field or prompt rather than free-form prose, and include prompts for allocation plans, limits, backpressure choices, pacing, batching, overload behavior, and open risks.
- [x] I3 Add exact Markdown links `[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)` and `[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)` to `README.md`, and add exact Markdown links `[Bounds and pacing review guide](bounds-and-pacing-review.md)`, `[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)`, and `[Safety spec contract](../openspec/specs/safety/spec.md)` to `docs/contributor-guide.md`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/bounds-and-pacing-review.md templates/bounds-and-pacing-checklist.md`; input=`draft guide plus copyable template`; asserts=`Tiger Style maintainer reviews the full guide and full template, confirms each required topic sits under its designated heading, confirms queue and event-source guidance includes explicit prompts for bounds, batching, backpressure, and pacing ownership, confirms audited exceptions and review cadence appear under ## Reviewer prompts and escape hatches, records reviewer role plus reviewed artifact paths naming docs/bounds-and-pacing-review.md and templates/bounds-and-pacing-checklist.md, and either (a) answers the four spec-defined questions with pass plus saves verdict: pass, or (b) records the failing answers with verdict: fail and sends the change back to I1-I3 before any V-task may pass`; evidence=`openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Guide keeps required headings and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/bounds-and-pacing-review.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Allocation lifetime and ownership',
    '## Startup sizing and post-init bounds',
    '## Queues, buffers, and backpressure',
    '## External-event pacing',
    '## Batch and work budgeting',
    '## Overload and failure behavior',
    '## Reviewer prompts and escape hatches',
]
assert headings == expected, f'unexpected guide headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-guide-headings.txt`
- [x] V3 Verify scenario "Template keeps required headings and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('templates/bounds-and-pacing-checklist.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Component or boundary',
    '## Allocation plan',
    '## Startup sizing and post-init limits',
    '## Queue and buffer bounds',
    '## External-event intake and pacing',
    '## Batch and work budget',
    '## Overload and failure behavior',
    '## Open risks and follow-ups',
]
assert headings == expected, f'unexpected template headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-template-headings.txt`
- [x] V4 Verify scenario "Reader finds the bounds guide from README" with `python3 - <<'PY'
from pathlib import Path
for path in ['docs/bounds-and-pacing-review.md', 'templates/bounds-and-pacing-checklist.md']:
    assert Path(path).is_file(), path
    print(f'exists:{path}')
lines = Path('README.md').read_text().splitlines()
for line in [
    '[Bounds and pacing review guide](docs/bounds-and-pacing-review.md)',
    '[Bounds and pacing template](templates/bounds-and-pacing-checklist.md)',
]:
    assert line in lines, line
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/readme-bounds-links.txt`
- [x] V5 Verify scenario "Contributor sees bounds doc touchpoints" with `python3 - <<'PY'
from pathlib import Path
for path in ['docs/bounds-and-pacing-review.md', 'templates/bounds-and-pacing-checklist.md', 'openspec/specs/safety/spec.md']:
    assert Path(path).is_file(), path
    print(f'exists:{path}')
lines = Path('docs/contributor-guide.md').read_text().splitlines()
for line in [
    '[Bounds and pacing review guide](bounds-and-pacing-review.md)',
    '[Bounds and pacing template](../templates/bounds-and-pacing-checklist.md)',
    '[Safety spec contract](../openspec/specs/safety/spec.md)',
]:
    assert line in lines, line
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/contributor-bounds-links.txt`
- [x] V6 Verify scenario "Saved review record keeps required labels" with `python3 - <<'PY'
from pathlib import Path
import re
path = Path('openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt')
text = path.read_text()
expected_lines = [
    'reviewer role: Tiger Style maintainer',
    'reviewed artifact paths: docs/bounds-and-pacing-review.md, templates/bounds-and-pacing-checklist.md',
    'question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading',
    'question 2: pass - guide covers every required bounds-and-pacing topic from Public Tiger Style Bounds And Pacing Review Guide under the designated headings',
    'question 3: pass - template covers every required field, including backpressure choices, and every section stays a fill-in field or prompt rather than free-form prose',
    'question 4: pass - the External-event pacing guidance says systems buffer external events and run at their own pace instead of reacting directly to every event',
    'verdict: pass',
]
for line in expected_lines:
    matches = re.findall(rf'^{re.escape(line)}$', text, flags=re.MULTILINE)
    assert len(matches) == 1, f'{line} count={len(matches)}'
    print(f'{line}:1')
PY`; save output at `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review-labels.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Guide keeps required headings and order` -> `V1` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-guide-headings.txt`
- `Scenario: Reviewing queue or event-source ownership` -> `H1` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
- `Scenario: Template keeps required headings and order` -> `V3` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-template-headings.txt`
- `Scenario: Planning allocation and pacing limits` -> `H1` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
- `Scenario: Reader finds the bounds guide from README` -> `V4` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/readme-bounds-links.txt`
- `Scenario: Contributor sees bounds doc touchpoints` -> `V5` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/contributor-bounds-links.txt`
- `Scenario: Saved guide heading artifact keeps exact heading inventory` -> `V1` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-guide-headings.txt`
- `Scenario: Saved template heading artifact keeps exact heading inventory` -> `V3` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-template-headings.txt`
- `Scenario: Saved README link artifact keeps existence and exact link lines` -> `V4` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/readme-bounds-links.txt`
- `Scenario: Saved contributor link artifact keeps existence and exact link lines` -> `V5` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/contributor-bounds-links.txt`
- `Scenario: Reviewer signs off bounds and pacing coverage` -> `H1` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
- `Scenario: Saved review-label artifact keeps exact label counts` -> `V6` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review-labels.txt`
- `Scenario: Saved review record keeps required labels` -> `V6` -> `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review-labels.txt`
