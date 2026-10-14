## Phase 1: Implementation

- [x] I1 Add `docs/safety-critical.md` with the exact guide headings from `openspec/changes/clarify-safety-critical-claims/specs/safety/spec.md`, explicit wording rules or maintainer prompts under every required heading, supported-claim guidance under `## What Tiger Style can claim`, unsupported-claim guidance under `## What Tiger Style cannot claim`, distinctions among lints/tests/proof/assurance/certification under `## Lints, tests, proof, assurance, and certification`, evidence and human-review expectations under `## Evidence and human review`, and concrete preferred/avoid phrasing under `## Responsible wording in docs and reviews`.
- [x] I2 Add the exact Markdown link `[Safety-critical claims guide](docs/safety-critical.md)` to `README.md`, and add the exact Markdown links `[Safety-critical claims guide](safety-critical.md)` and `[Safety spec contract](../openspec/specs/safety/spec.md)` to `docs/contributor-guide.md`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/safety-critical.md`; input=`draft safety-critical claims guide`; asserts=`Tiger Style maintainer reviews the full guide, writes the exact metadata lines `reviewer role: Tiger Style maintainer` and `reviewed artifact paths: docs/safety-critical.md` into openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt, confirms every required topic sits under its designated heading, confirms the guide distinguishes supported Tiger Style claims from unsupported certification or assurance claims, confirms the guide says high-consequence claims need named evidence and human review beyond green automation, confirms every required section includes at least one explicit maintainer prompt or wording rule, confirms the guide gives explicit wording guidance for docs, reviews, and rollout notes, and either (a) answers the five spec-defined questions with pass plus saves verdict: pass, or (b) records the failing answers with verdict: fail using the same exact labels and sends the change back to I1-I2 before any V-task may pass`; evidence=`openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Reader sees what Tiger Style can and cannot claim" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/safety-critical.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## What Tiger Style can claim',
    '## What Tiger Style cannot claim',
    '## Lints, tests, proof, assurance, and certification',
    '## Evidence and human review',
    '## Responsible wording in docs and reviews',
]
assert headings == expected, headings
print('\n'.join(headings))
PY`; save output at `openspec/changes/clarify-safety-critical-claims/evidence/safety-critical-guide-headings.txt`
- [x] V2 Verify scenario "Reader finds the safety-critical guide from README" with `python3 - <<'PY'
from pathlib import Path
readme = Path('README.md').read_text().splitlines()
assert Path('docs/safety-critical.md').is_file()
line = '[Safety-critical claims guide](docs/safety-critical.md)'
assert line in readme, line
print('exists:docs/safety-critical.md')
print(line)
PY`; save output at `openspec/changes/clarify-safety-critical-claims/evidence/readme-safety-critical-links.txt`
- [x] V3 Verify scenario "Contributor sees safety-critical doc touchpoints" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/contributor-guide.md').read_text().splitlines()
assert Path('docs/safety-critical.md').is_file()
assert Path('openspec/specs/safety/spec.md').is_file()
expected = [
    '[Safety-critical claims guide](safety-critical.md)',
    '[Safety spec contract](../openspec/specs/safety/spec.md)',
]
for line in expected:
    assert line in lines, line
print('exists:docs/safety-critical.md')
print('exists:openspec/specs/safety/spec.md')
print('\n'.join(expected))
PY`; save output at `openspec/changes/clarify-safety-critical-claims/evidence/contributor-safety-critical-links.txt`
- [x] V4 Verify scenario "Saved review record keeps required labels" with `python3 - <<'PY'
from pathlib import Path
from collections import Counter
path = Path('openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt')
lines = path.read_text().splitlines()
required = [
    'reviewer role: Tiger Style maintainer',
    'reviewed artifact paths: docs/safety-critical.md',
    'question 1: pass - guide distinguishes clearly between supported Tiger Style claims and unsupported certification or assurance claims',
    'question 2: pass - guide explains the difference among lints, tests, formal proof, assurance evidence, and certification without overstating any one surface',
    'question 3: pass - guide says high-consequence claims need named evidence and human review beyond green automation',
    'question 4: pass - every required section includes at least one explicit maintainer prompt or wording rule',
    'question 5: pass - guide gives explicit wording guidance for docs, reviews, and rollout notes so maintainers do not imply unsupported safety-critical guarantees',
    'verdict: pass',
]
counts = Counter(lines)
for line in required:
    assert counts[line] == 1, (line, counts[line])
for line in required:
    print(f'{line}:{counts[line]}')
PY`; save output at `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review-labels.txt`

## Change Status
- Status: completed
- Open tasks: none
## Verification Coverage

- `Scenario: Reader sees what Tiger Style can and cannot claim` -> `V1` -> `openspec/changes/clarify-safety-critical-claims/evidence/safety-critical-guide-headings.txt`
- `Scenario: Saved guide heading artifact keeps exact heading inventory` -> `V1` -> `openspec/changes/clarify-safety-critical-claims/evidence/safety-critical-guide-headings.txt`
- `Scenario: Reader finds the safety-critical guide from README` -> `V2` -> `openspec/changes/clarify-safety-critical-claims/evidence/readme-safety-critical-links.txt`
- `Scenario: Saved README link artifact keeps existence and exact link line` -> `V2` -> `openspec/changes/clarify-safety-critical-claims/evidence/readme-safety-critical-links.txt`
- `Scenario: Contributor sees safety-critical doc touchpoints` -> `V3` -> `openspec/changes/clarify-safety-critical-claims/evidence/contributor-safety-critical-links.txt`
- `Scenario: Saved contributor link artifact keeps existence and exact link lines` -> `V3` -> `openspec/changes/clarify-safety-critical-claims/evidence/contributor-safety-critical-links.txt`
- `Scenario: Reviewer signs off claims language` -> `H1` -> `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`
- `Scenario: Saved review-label artifact keeps exact label counts` -> `V4` -> `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review-labels.txt`
- `Scenario: Saved review record keeps required labels` -> `V4` -> `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review-labels.txt`
