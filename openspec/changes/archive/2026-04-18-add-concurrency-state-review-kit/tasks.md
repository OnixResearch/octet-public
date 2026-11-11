## Phase 1: Implementation

- [x] I1 Add `docs/concurrency-state-review.md` with the exact guide headings from `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/specs/concurrency/spec.md`, reviewer prompts under every required heading, explanation of why each section matters, explicit guidance on shared-state inventory, ordering rules, transition invariants, retry/replay assumptions, unsafe quarantine, failure handling and escalation, and overall guide guidance that unsafe stays behind narrow safe wrappers, dangerous escape hatches stay grep-able and explicit, and this document is a review kit rather than the full enforcement story.
- [x] I2 Add `templates/concurrency-state-checklist.md` with the exact template headings from `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/specs/concurrency/spec.md`, make every section a fill-in field or prompt rather than free-form prose, and include prompts for shared-state inventory, ordering, transitions, retry/idempotence assumptions, unsafe boundaries, and open risks.
- [x] I3 Add exact Markdown links `[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)` and `[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)` to `README.md`, and add exact Markdown links `[Concurrency and distributed-state review guide](concurrency-state-review.md)`, `[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)`, and `[Concurrency spec contract](../openspec/specs/concurrency/spec.md)` to `docs/contributor-guide.md`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/concurrency-state-review.md templates/concurrency-state-checklist.md openspec/specs/concurrency/spec.md`; input=`draft guide plus copyable template plus canonical concurrency spec`; asserts=`Tiger Style maintainer reviews the full guide, full template, and canonical concurrency spec, confirms each required topic sits under its designated heading, confirms each required guide heading explains why that section matters, confirms ordering and retry/replay guidance stay explicit, confirms failure-handling and escalation guidance appears under ## Reviewer prompts and failure handling, confirms the docs stay aligned with openspec/specs/concurrency/spec.md, records reviewer role plus reviewed artifact paths naming docs/concurrency-state-review.md and templates/concurrency-state-checklist.md, and either (a) answers the four spec-defined questions with pass plus saves verdict: pass, or (b) records the failing answers with verdict: fail and sends the change back to I1-I3 before any V-task may pass`; evidence=`openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Guide keeps required headings and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/concurrency-state-review.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Shared-state inventory and ownership',
    '## Lock ordering and peer ordering',
    '## State-machine transitions and invariants',
    '## Message ordering, retries, and idempotence',
    '## Unsafe quarantine and escape hatches',
    '## Reviewer prompts and failure handling',
]
assert headings == expected, f'unexpected guide headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-guide-headings.txt`
- [x] V3 Verify scenario "Template keeps required headings and order" with `python3 - <<'PY'
from pathlib import Path
lines = Path('templates/concurrency-state-checklist.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Component or protocol boundary',
    '## Shared-state inventory',
    '## Lock and peer ordering',
    '## State-machine transitions and invariants',
    '## Delivery, retry, and idempotence assumptions',
    '## Unsafe boundary and escape hatches',
    '## Open risks and follow-ups',
]
assert headings == expected, f'unexpected template headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-template-headings.txt`
- [x] V4 Verify scenario "Reader finds the concurrency guide from README" with `python3 - <<'PY'
from pathlib import Path
for path in ['docs/concurrency-state-review.md', 'templates/concurrency-state-checklist.md']:
    assert Path(path).is_file(), path
    print(f'exists:{path}')
lines = Path('README.md').read_text().splitlines()
for line in [
    '[Concurrency and distributed-state review guide](docs/concurrency-state-review.md)',
    '[Concurrency and distributed-state template](templates/concurrency-state-checklist.md)',
]:
    assert line in lines, line
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/readme-concurrency-links.txt`
- [x] V5 Verify scenario "Contributor sees concurrency doc touchpoints" with `python3 - <<'PY'
from pathlib import Path
for path in ['docs/concurrency-state-review.md', 'templates/concurrency-state-checklist.md', 'openspec/specs/concurrency/spec.md']:
    assert Path(path).is_file(), path
    print(f'exists:{path}')
lines = Path('docs/contributor-guide.md').read_text().splitlines()
for line in [
    '[Concurrency and distributed-state review guide](concurrency-state-review.md)',
    '[Concurrency and distributed-state template](../templates/concurrency-state-checklist.md)',
    '[Concurrency spec contract](../openspec/specs/concurrency/spec.md)',
]:
    assert line in lines, line
    print(line)
PY`; save output at `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/contributor-concurrency-links.txt`
- [x] V6 Verify scenario "Saved review record keeps required labels" with `python3 - <<'PY'
from pathlib import Path
import re
path = Path('openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt')
text = path.read_text()
expected_lines = [
    'reviewer role: Tiger Style maintainer',
    'reviewed artifact paths: docs/concurrency-state-review.md, templates/concurrency-state-checklist.md',
    'question 1: pass - each required guide heading explains why that section matters and includes reviewer prompts',
    'question 2: pass - guide covers every required concurrency topic from Public Tiger Style Concurrency And Distributed-State Review Guide under the designated headings and stays aligned with openspec/specs/concurrency/spec.md',
    'question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose',
    'question 4: pass - overall guide guidance says unsafe stays behind narrow safe wrappers, dangerous escape hatches stay grep-able and explicit, and this document is a review kit rather than the full enforcement story',
    'verdict: pass',
]
for line in expected_lines:
    matches = re.findall(rf'^{re.escape(line)}$', text, flags=re.MULTILINE)
    assert len(matches) == 1, f'{line} count={len(matches)}'
    print(f'{line}:1')
PY`; save output at `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review-labels.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Guide keeps required headings and order` -> `V1` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-guide-headings.txt`
- `Scenario: Reviewing lock order or protocol transitions` -> `H1` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
- `Scenario: Template keeps required headings and order` -> `V3` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-template-headings.txt`
- `Scenario: Planning ordering and retry assumptions` -> `H1` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
- `Scenario: Reader finds the concurrency guide from README` -> `V4` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/readme-concurrency-links.txt`
- `Scenario: Contributor sees concurrency doc touchpoints` -> `V5` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/contributor-concurrency-links.txt`
- `Scenario: Saved guide heading artifact keeps exact heading inventory` -> `V1` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-guide-headings.txt`
- `Scenario: Saved template heading artifact keeps exact heading inventory` -> `V3` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/concurrency-template-headings.txt`
- `Scenario: Saved README link artifact keeps existence and exact link lines` -> `V4` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/readme-concurrency-links.txt`
- `Scenario: Saved contributor link artifact keeps existence and exact link lines` -> `V5` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/contributor-concurrency-links.txt`
- `Scenario: Reviewer signs off concurrency coverage` -> `H1` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review.txt`
- `Scenario: Saved review-label artifact keeps exact label counts` -> `V6` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review-labels.txt`
- `Scenario: Saved review record keeps required labels` -> `V6` -> `openspec/changes/archive/2026-04-18-add-concurrency-state-review-kit/evidence/human-concurrency-review-labels.txt`
