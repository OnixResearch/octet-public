## Phase 1: Implementation

- [x] I1 Add `docs/system-comprehension.md` with the exact guide headings from `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/specs/architecture/spec.md`, explicit reviewer prompts under every required heading, audience/slice guidance under `## Reader, boundary, and system slice`, major-component mapping under `## Structure before function`, end-to-end explanation guidance under `## Functional walkthrough after structure`, weak-spot and ownership guidance under `## Deep dives, weak spots, and ownership`, top-level doc and crate-root guidance under `## Top-level docs, crate roots, and dependency maps`, and a 2-minute subsystem-map review rule under `## Reviewer prompts for 2-minute legibility`.
- [x] I2 Add `templates/system-walkthrough.md` with the exact template headings from `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/specs/architecture/spec.md`, and keep every section a fill-in field or prompt rather than free-form prose.
- [x] I3 Add the exact Markdown links `[System comprehension guide](docs/system-comprehension.md)` and `[System walkthrough template](templates/system-walkthrough.md)` to `README.md`, and add the exact Markdown links `[System comprehension guide](system-comprehension.md)` and `[System walkthrough template](../templates/system-walkthrough.md)` to `docs/contributor-guide.md`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/system-comprehension.md templates/system-walkthrough.md`; input=`draft guide plus walkthrough template`; asserts=`Tiger Style maintainer reviews the full guide and full template, writes the exact metadata lines `reviewer role: Tiger Style maintainer` and `reviewed artifact paths: docs/system-comprehension.md, templates/system-walkthrough.md` into openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt, confirms the guide teaches structure before function before depth, confirms the guide covers reader, boundary, system slice, one primary end-to-end flow, and selected deep dives with ownership gaps, confirms top-level docs, crate roots, and dependency maps are treated as comprehension surfaces, confirms every required guide heading includes at least one explicit reviewer prompt or checklist question, confirms every template section stays a fill-in prompt rather than free-form prose, and either (a) answers the five spec-defined questions with pass plus saves verdict: pass, or (b) records the failing answers with verdict: fail using the same exact labels and sends the change back to I1-I3 before any V-task may pass`; evidence=`openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Guide teaches structure before function before depth" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/system-comprehension.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Reader, boundary, and system slice',
    '## Structure before function',
    '## Functional walkthrough after structure',
    '## Deep dives, weak spots, and ownership',
    '## Top-level docs, crate roots, and dependency maps',
    '## Reviewer prompts for 2-minute legibility',
]
assert headings == expected, headings
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/system-comprehension-guide-headings.txt`
- [x] V2 Verify scenario "Template captures a guided system walkthrough" with `python3 - <<'PY'
from pathlib import Path
lines = Path('templates/system-walkthrough.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Intended reader and system slice',
    '## Major components and boundaries',
    '## Key relationships and dependency edges',
    '## Primary end-to-end flow',
    '## Deep dives and weak spots',
    '## Open questions and follow-ups',
]
assert headings == expected, headings
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/system-walkthrough-template-headings.txt`
- [x] V3 Verify scenario "Reader finds the system comprehension guide from README" with `python3 - <<'PY'
from pathlib import Path
readme = Path('README.md').read_text().splitlines()
assert Path('docs/system-comprehension.md').is_file()
assert Path('templates/system-walkthrough.md').is_file()
expected = [
    '[System comprehension guide](docs/system-comprehension.md)',
    '[System walkthrough template](templates/system-walkthrough.md)',
]
for line in expected:
    assert line in readme, line
print('exists:docs/system-comprehension.md')
print('exists:templates/system-walkthrough.md')
print('\n'.join(expected))
PY`; save output at `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/readme-system-comprehension-links.txt`
- [x] V4 Verify scenario "Contributor sees system-comprehension doc touchpoints" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/contributor-guide.md').read_text().splitlines()
assert Path('docs/system-comprehension.md').is_file()
assert Path('templates/system-walkthrough.md').is_file()
expected = [
    '[System comprehension guide](system-comprehension.md)',
    '[System walkthrough template](../templates/system-walkthrough.md)',
]
for line in expected:
    assert line in lines, line
print('exists:docs/system-comprehension.md')
print('exists:templates/system-walkthrough.md')
print('\n'.join(expected))
PY`; save output at `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/contributor-system-comprehension-links.txt`
- [x] V5 Verify scenario "Saved review record keeps required labels" with `python3 - <<'PY'
from pathlib import Path
from collections import Counter
path = Path('openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt')
lines = path.read_text().splitlines()
required = [
    'reviewer role: Tiger Style maintainer',
    'reviewed artifact paths: docs/system-comprehension.md, templates/system-walkthrough.md',
    'question 1: pass - guide teaches readers to explain structure before function before depth',
    'question 2: pass - guide covers reader, boundary, system slice, one primary end-to-end flow, and selected deep dives with ownership gaps',
    'question 3: pass - guide makes top-level docs, crate roots, and dependency maps part of comprehension review',
    'question 4: pass - every required guide heading includes at least one explicit reviewer prompt or checklist question',
    'question 5: pass - template covers every required section and each section stays a fill-in prompt rather than free-form prose',
    'verdict: pass',
]
counts = Counter(lines)
for line in required:
    assert counts[line] == 1, (line, counts[line])
for line in required:
    print(f'{line}:{counts[line]}')
PY`; save output at `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review-labels.txt`

## Change Status
- Status: done
- Open tasks: none
## Verification Coverage

- `Scenario: Guide teaches structure before function before depth` -> `V1` -> `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/system-comprehension-guide-headings.txt`
- `Scenario: Template captures a guided system walkthrough` -> `V2` -> `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/system-walkthrough-template-headings.txt`
- `Scenario: Reader finds the system comprehension guide from README` -> `V3` -> `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/readme-system-comprehension-links.txt`
- `Scenario: Contributor sees system-comprehension doc touchpoints` -> `V4` -> `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/contributor-system-comprehension-links.txt`
- `Scenario: Reviewer signs off system-comprehension guidance quality` -> `H1` -> `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`
- `Scenario: Saved review record keeps required labels` -> `V5` -> `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review-labels.txt`
