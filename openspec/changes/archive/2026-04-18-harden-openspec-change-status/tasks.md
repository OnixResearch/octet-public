## Phase 1: Implementation

- [x] I1 Update `openspec/templates/tasks.md` so it adds the exact `## Change Status` section from `openspec/changes/harden-openspec-change-status/specs/openspec-task-governance/spec.md` before `## Verification Coverage`, keeps the exact example lines `- Status: in-progress` and `- Open tasks: I1, I2, V1, V2, H1`, and says `Open tasks: none` is only valid when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
- [x] I2 Update repo-local task-stage gate guidance or logic in `openspec/config.yaml` and supporting checks so active `openspec/changes/*/tasks.md` files require `## Change Status` immediately before `## Verification Coverage`, drive status only from structured `I<n>`, `V<n>`, and `H<n>` task checkboxes, accept only the exact `in-progress` / listed-open-tasks shape or the exact `complete` / `Open tasks: none` shape, derive unchecked task IDs from checkbox state, and block the exact finding strings from `openspec/changes/harden-openspec-change-status/specs/openspec-task-governance/spec.md`.
- [x] I3 Add fixtures under `tests/fixtures/openspec-task-governance/` for `change-status-in-progress-pass`, `change-status-complete-pass`, `change-status-ignores-unstructured-checkboxes-pass`, `missing-change-status-blocked`, `malformed-or-misplaced-change-status-blocked`, `complete-status-with-open-tasks-blocked`, `in-progress-without-open-tasks-blocked`, and `open-task-list-mismatch-blocked`.
- [x] I4 Add `tests/fixtures/openspec-task-governance/run-fixture-gate.sh` so it copies one named fixture into `openspec/changes/<fixture-name>/`, prints the full self-describing command preamble `tests/fixtures/openspec-task-governance/run-fixture-gate.sh <fixture-name>`, runs the tasks-stage gate for that temporary active change, and removes the temporary change directory afterward.
- [x] I5 Update `openspec/changes/add-performance-review-kit/tasks.md`, `openspec/changes/add-assertion-review-kit/tasks.md`, `openspec/changes/add-bounds-and-pacing-review-kit/tasks.md`, `openspec/changes/add-concurrency-state-review-kit/tasks.md`, `openspec/changes/clarify-safety-critical-claims/tasks.md`, `openspec/changes/add-system-comprehension-guidance/tasks.md`, and `openspec/changes/add-resilience-review-kit/tasks.md` so each active sibling file gains a compliant `## Change Status` section before `## Verification Coverage` with an `Open tasks` line that matches its current unchecked task IDs.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Template shows change-status fields" with `python3 - <<'PY'
from pathlib import Path
import re
lines = Path('openspec/templates/tasks.md').read_text().splitlines()
start = lines.index('## Change Status')
snippet = lines[start:start + 3]
expected = [
    '## Change Status',
    '- Status: in-progress',
    '- Open tasks: I1, I2, V1, V2, H1',
]
assert snippet == expected, snippet
note = 'Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.'
assert note in lines[start + 3], lines[start + 3]
assert lines[start + 4] == '## Verification Coverage', lines[start + 4]
checkbox_ids = []
for line in lines[:start]:
    match = re.match(r'- \[(?: |x)\] ((?:I|V|H)\d+)\b', line)
    if match:
        checkbox_ids.append(match.group(1))
assert checkbox_ids == ['I1', 'I2', 'V1', 'V2', 'H1'], checkbox_ids
print('\n'.join(snippet))
print(note)
print(lines[start + 4])
PY`; save output at `openspec/changes/harden-openspec-change-status/evidence/change-status-template-snippet.txt`
- [x] V2 Verify scenario "Unchecked tasks keep change in progress" with `python3 - <<'PY'
from pathlib import Path
import re
lines = Path('tests/fixtures/openspec-task-governance/change-status-in-progress-pass/tasks.md').read_text().splitlines()
start = lines.index('## Change Status')
snippet = lines[start:start + 3]
expected = [
    '## Change Status',
    '- Status: in-progress',
    '- Open tasks: I2, V1, H1',
]
assert snippet == expected, snippet
assert lines[start + 3] == '## Verification Coverage', lines[start + 3]
unchecked = []
for line in lines[:start]:
    match = re.match(r'- \[ \] ((?:I|V|H)\d+)\b', line)
    if match:
        unchecked.append(match.group(1))
assert unchecked == ['I2', 'V1', 'H1'], unchecked
print('\n'.join(snippet))
print(lines[start + 3])
PY`; save output at `openspec/changes/harden-openspec-change-status/evidence/change-status-in-progress-pass.txt`
- [x] V3 Verify scenario "Complete status is only valid when all tasks are checked" with `python3 - <<'PY'
from pathlib import Path
import re
lines = Path('tests/fixtures/openspec-task-governance/change-status-complete-pass/tasks.md').read_text().splitlines()
start = lines.index('## Change Status')
snippet = lines[start:start + 3]
expected = [
    '## Change Status',
    '- Status: complete',
    '- Open tasks: none',
]
assert snippet == expected, snippet
assert lines[start + 3] == '## Verification Coverage', lines[start + 3]
checkboxes = []
for line in lines[:start]:
    match = re.match(r'- \[(?: |x)\] ((?:I|V|H)\d+)\b', line)
    if match:
        checkboxes.append(line)
assert checkboxes and all(line.startswith('- [x] ') for line in checkboxes), checkboxes
print('\n'.join(snippet))
print(lines[start + 3])
PY`; save output at `openspec/changes/harden-openspec-change-status/evidence/change-status-complete-pass.txt`
- [x] V4 Verify scenario "Unstructured checkboxes do not change status" with `python3 - <<'PY'
from pathlib import Path
import re
lines = Path('tests/fixtures/openspec-task-governance/change-status-ignores-unstructured-checkboxes-pass/tasks.md').read_text().splitlines()
start = lines.index('## Change Status')
snippet = lines[start:start + 3]
expected = [
    '## Change Status',
    '- Status: in-progress',
    '- Open tasks: I2, V1',
]
assert snippet == expected, snippet
assert lines[start + 3] == '## Verification Coverage', lines[start + 3]
unchecked = []
for line in lines[:start]:
    match = re.match(r'- \[ \] ((?:I|V|H)\d+)\b', line)
    if match:
        unchecked.append(match.group(1))
assert unchecked == ['I2', 'V1'], unchecked
print('\n'.join(snippet))
print(lines[start + 3])
PY`; save output at `openspec/changes/harden-openspec-change-status/evidence/unstructured-checkboxes-ignored-pass.txt`
- [x] V5 Verify scenario "Missing change status is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked`; assert the saved output contains both `tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked` and `tasks.md is missing ## Change Status`; save output at `openspec/changes/harden-openspec-change-status/evidence/missing-change-status.txt`
- [x] V6 Verify scenario "Malformed or misplaced change status is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh malformed-or-misplaced-change-status-blocked`; assert the saved output contains both `tests/fixtures/openspec-task-governance/run-fixture-gate.sh malformed-or-misplaced-change-status-blocked` and `Change status section is malformed or misplaced`; save output at `openspec/changes/harden-openspec-change-status/evidence/malformed-or-misplaced-change-status.txt`
- [x] V7 Verify scenario "Complete status with open tasks is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh complete-status-with-open-tasks-blocked`; assert the saved output contains both `tests/fixtures/openspec-task-governance/run-fixture-gate.sh complete-status-with-open-tasks-blocked` and `Change status says complete but unchecked tasks remain`; save output at `openspec/changes/harden-openspec-change-status/evidence/complete-status-blocked.txt`
- [x] V8 Verify scenario "In-progress status with no open tasks is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh in-progress-without-open-tasks-blocked`; assert the saved output contains both `tests/fixtures/openspec-task-governance/run-fixture-gate.sh in-progress-without-open-tasks-blocked` and `Change status says in-progress but no open tasks remain`; save output at `openspec/changes/harden-openspec-change-status/evidence/in-progress-without-open-tasks-blocked.txt`
- [x] V9 Verify scenario "Incorrect open-task list is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh open-task-list-mismatch-blocked`; assert the saved output contains both `tests/fixtures/openspec-task-governance/run-fixture-gate.sh open-task-list-mismatch-blocked` and `Change status open-task list does not match unchecked tasks`; save output at `openspec/changes/harden-openspec-change-status/evidence/open-task-list-mismatch.txt`
- [x] V10 Verify scenario "Existing active changes stay compliant during rollout" with `python3 - <<'PY'
from pathlib import Path
import re
checks = [
    ('openspec/changes/add-performance-review-kit/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6'),
    ('openspec/changes/add-assertion-review-kit/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6'),
    ('openspec/changes/add-bounds-and-pacing-review-kit/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6'),
    ('openspec/changes/add-concurrency-state-review-kit/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6'),
    ('openspec/changes/clarify-safety-critical-claims/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, H1, V1, V2, V3, V4'),
    ('openspec/changes/add-system-comprehension-guidance/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, I3, H1, V1, V2, V3, V4, V5'),
    ('openspec/changes/add-resilience-review-kit/tasks.md', '- Status: in-progress', '- Open tasks: I1, I2, I3, H1, V1, V2, V3, V4, V5'),
]
for path_str, status_line, open_line in checks:
    lines = Path(path_str).read_text().splitlines()
    start = lines.index('## Change Status')
    assert lines[start + 1] == status_line, (path_str, lines[start + 1], status_line)
    assert lines[start + 2] == open_line, (path_str, lines[start + 2], open_line)
    assert lines[start + 3] == '## Verification Coverage', (path_str, lines[start + 3])
    unchecked = []
    for line in lines[:start]:
        match = re.match(r'- \[ \] ((?:I|V|H)\d+)\b', line)
        if match:
            unchecked.append(match.group(1))
    assert open_line == f"- Open tasks: {', '.join(unchecked)}", (path_str, unchecked)
    print(f'file:{path_str}')
    print(status_line)
    print(open_line)
    print(lines[start + 3])
PY`; save output at `openspec/changes/harden-openspec-change-status/evidence/active-change-rollout-status.txt`
- [x] V11 Verify helper cleanup with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked >/tmp/harden-openspec-change-status-helper-cleanup.txt && test ! -e openspec/changes/missing-change-status-blocked && printf '%s\n' 'tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked' 'removed:openspec/changes/missing-change-status-blocked'`; save output at `openspec/changes/harden-openspec-change-status/evidence/helper-cleanup.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Template shows change-status fields` -> `V1` -> `openspec/changes/harden-openspec-change-status/evidence/change-status-template-snippet.txt`
- `Scenario: Unchecked tasks keep change in progress` -> `V2` -> `openspec/changes/harden-openspec-change-status/evidence/change-status-in-progress-pass.txt`
- `Scenario: Complete status is only valid when all tasks are checked` -> `V3` -> `openspec/changes/harden-openspec-change-status/evidence/change-status-complete-pass.txt`
- `Scenario: Unstructured checkboxes do not change status` -> `V4` -> `openspec/changes/harden-openspec-change-status/evidence/unstructured-checkboxes-ignored-pass.txt`
- `Scenario: Missing change status is blocked` -> `V5` -> `openspec/changes/harden-openspec-change-status/evidence/missing-change-status.txt`
- `Scenario: Malformed or misplaced change status is blocked` -> `V6` -> `openspec/changes/harden-openspec-change-status/evidence/malformed-or-misplaced-change-status.txt`
- `Scenario: Complete status with open tasks is blocked` -> `V7` -> `openspec/changes/harden-openspec-change-status/evidence/complete-status-blocked.txt`
- `Scenario: In-progress status with no open tasks is blocked` -> `V8` -> `openspec/changes/harden-openspec-change-status/evidence/in-progress-without-open-tasks-blocked.txt`
- `Scenario: Incorrect open-task list is blocked` -> `V9` -> `openspec/changes/harden-openspec-change-status/evidence/open-task-list-mismatch.txt`
- `Scenario: Existing active changes stay compliant during rollout` -> `V10` -> `openspec/changes/harden-openspec-change-status/evidence/active-change-rollout-status.txt`
- `Scenario: Helper cleanup check` -> `V11` -> `openspec/changes/harden-openspec-change-status/evidence/helper-cleanup.txt`
