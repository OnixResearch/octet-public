## Phase 1: Implementation

- [x] I1 Add `docs/resilience-review.md` with the exact guide headings from `openspec/changes/archive/2026-04-18-add-resilience-review-kit/specs/resilience/spec.md`, explicit reviewer prompts under every required heading, incident and workaround framing under `## Incident, symptom, and immediate workaround`, symptom-to-design-cause guidance under `## Five whys to the design decision`, blast-radius and invariant guidance under `## Boundary, blast radius, and affected invariants`, retry/replay/idempotence/degradation guidance under `## Retry, replay, idempotence, and degradation`, explicit failure-posture guidance under `## Fail-open vs fail-closed`, and follow-up ownership plus the rule that restart or retry alone is not root cause under `## Reviewer prompts and follow-ups`.
- [x] I2 Add `templates/five-whys-architecture.md` with the exact template headings from `openspec/changes/archive/2026-04-18-add-resilience-review-kit/specs/resilience/spec.md`, and keep every section a fill-in field or prompt rather than free-form prose.
- [x] I3 Add the exact Markdown links `[Resilience review guide](docs/resilience-review.md)` and `[Five-whys architecture template](templates/five-whys-architecture.md)` to `README.md`, and add the exact Markdown links `[Resilience review guide](resilience-review.md)` and `[Five-whys architecture template](../templates/five-whys-architecture.md)` to `docs/contributor-guide.md`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/resilience-review.md templates/five-whys-architecture.md`; input=`draft resilience guide plus five-whys template`; asserts=`Tiger Style maintainer reviews the full guide and full template, writes the exact metadata lines `reviewer role: Tiger Style maintainer` and `reviewed artifact paths: docs/resilience-review.md, templates/five-whys-architecture.md` into openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt, confirms the guide climbs from symptom and workaround to a design decision rather than stopping at retry or restart, confirms the guide makes blast radius, affected invariants, retry or replay assumptions, degradation posture, and fail-open versus fail-closed choice explicit, confirms the guide requires follow-up ownership and the rule that restart or retry alone is not root cause, confirms every required guide heading includes at least one explicit reviewer prompt or checklist question, confirms every template section stays a fill-in prompt rather than free-form prose, and either (a) answers the five spec-defined questions with pass plus saves verdict: pass, or (b) records the failing answers with verdict: fail using the same exact labels and sends the change back to I1-I3 before any V-task may pass`; evidence=`openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Guide traces a recurring incident to a design decision" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/resilience-review.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Incident, symptom, and immediate workaround',
    '## Five whys to the design decision',
    '## Boundary, blast radius, and affected invariants',
    '## Retry, replay, idempotence, and degradation',
    '## Fail-open vs fail-closed',
    '## Reviewer prompts and follow-ups',
]
assert headings == expected, headings
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/resilience-guide-headings.txt`
- [x] V2 Verify scenario "Template captures a five-whys architecture review" with `python3 - <<'PY'
from pathlib import Path
lines = Path('templates/five-whys-architecture.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Incident or scenario under review',
    '## Symptom and immediate workaround',
    '## Five whys chain',
    '## Boundary, blast radius, and affected invariants',
    '## Retry, replay, idempotence, and degradation assumptions',
    '## Fail-open vs fail-closed decision',
    '## Follow-ups, owners, and non-goals',
]
assert headings == expected, headings
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/five-whys-template-headings.txt`
- [x] V3 Verify scenario "Reader finds the resilience guide from README" with `python3 - <<'PY'
from pathlib import Path
readme = Path('README.md').read_text().splitlines()
assert Path('docs/resilience-review.md').is_file()
assert Path('templates/five-whys-architecture.md').is_file()
expected = [
    '[Resilience review guide](docs/resilience-review.md)',
    '[Five-whys architecture template](templates/five-whys-architecture.md)',
]
for line in expected:
    assert line in readme, line
print('exists:docs/resilience-review.md')
print('exists:templates/five-whys-architecture.md')
print('\n'.join(expected))
PY`; save output at `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/readme-resilience-links.txt`
- [x] V4 Verify scenario "Contributor sees resilience doc touchpoints" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/contributor-guide.md').read_text().splitlines()
assert Path('docs/resilience-review.md').is_file()
assert Path('templates/five-whys-architecture.md').is_file()
expected = [
    '[Resilience review guide](resilience-review.md)',
    '[Five-whys architecture template](../templates/five-whys-architecture.md)',
]
for line in expected:
    assert line in lines, line
print('exists:docs/resilience-review.md')
print('exists:templates/five-whys-architecture.md')
print('\n'.join(expected))
PY`; save output at `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/contributor-resilience-links.txt`
- [x] V5 Verify scenario "Saved review record keeps required labels" with `python3 - <<'PY'
from pathlib import Path
from collections import Counter
path = Path('openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt')
lines = path.read_text().splitlines()
required = [
    'reviewer role: Tiger Style maintainer',
    'reviewed artifact paths: docs/resilience-review.md, templates/five-whys-architecture.md',
    'question 1: pass - guide climbs from symptom and workaround to a design decision rather than stopping at retry or restart',
    'question 2: pass - guide makes blast radius, affected invariants, retry or replay assumptions, and degradation posture explicit',
    'question 3: pass - guide requires an explicit fail-open versus fail-closed decision, follow-up ownership, and the rule that restart or retry alone is not root cause',
    'question 4: pass - every required guide heading includes at least one explicit reviewer prompt or checklist question',
    'question 5: pass - template covers every required section and each section stays a fill-in prompt rather than free-form prose',
    'verdict: pass',
]
counts = Counter(lines)
for line in required:
    assert counts[line] == 1, (line, counts[line])
for line in required:
    print(f'{line}:{counts[line]}')
PY`; save output at `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review-labels.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Guide traces a recurring incident to a design decision` -> `V1` -> `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/resilience-guide-headings.txt`
- `Scenario: Template captures a five-whys architecture review` -> `V2` -> `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/five-whys-template-headings.txt`
- `Scenario: Reader finds the resilience guide from README` -> `V3` -> `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/readme-resilience-links.txt`
- `Scenario: Contributor sees resilience doc touchpoints` -> `V4` -> `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/contributor-resilience-links.txt`
- `Scenario: Reviewer signs off resilience guidance quality` -> `H1` -> `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`
- `Scenario: Saved review record keeps required labels` -> `V5` -> `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review-labels.txt`
