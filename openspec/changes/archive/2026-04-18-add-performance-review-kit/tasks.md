## Phase 1: Implementation

- [x] I1 Add `docs/performance-review.md` with the exact guide headings from `openspec/changes/archive/2026-04-18-add-performance-review-kit/specs/performance/spec.md`, reviewer prompts under every required heading, explanation of why the checklist matters, early design-before-profiling guidance under `## Workload and hot path`, network/disk/memory/CPU plus bandwidth/latency reasoning under `## Resource sketch: network, disk, memory, CPU`, measurement/profiling handoff guidance under `## Measurement and validation`, large-work-chunk guidance under `## CPU predictability and compiler luck`, and overall guide guidance stating that static lints and benchmarks do not replace workload reasoning.
- [x] I2 Add `templates/performance-sketch.md` with the exact template headings from `openspec/changes/archive/2026-04-18-add-performance-review-kit/specs/performance/spec.md`, make every section a fill-in field or prompt rather than free-form prose, and include prompts for workload, assumptions, bandwidth/latency math, resource budgets or limits per relevant resource, cost × frequency, control/data-plane split, batching/backpressure, validation, and open risks.
- [x] I3 Add exact Markdown links `[Performance review guide](docs/performance-review.md)` and `[Performance sketch template](templates/performance-sketch.md)` to `README.md`, and add exact Markdown links `[Performance review guide](performance-review.md)`, `[Performance sketch template](../templates/performance-sketch.md)`, and `[Performance spec contract](../openspec/specs/performance/spec.md)` to `docs/contributor-guide.md`.

## Phase 2: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat docs/performance-review.md templates/performance-sketch.md`; input=`draft guide plus copyable template`; asserts=`Tiger Style maintainer reviews the full guide and full template, confirms each required topic sits under its designated heading, confirms the queue/buffer-growth guidance includes explicit prompts for bounds, batching, and cost × frequency reasoning, confirms the static-lints/benchmarks disclaimer appears in overall guide guidance rather than per-section boilerplate, records reviewer role plus reviewed artifact paths naming docs/performance-review.md and templates/performance-sketch.md, and either (a) answers the four spec-defined questions with pass plus saves verdict: pass, or (b) records the failing answers with verdict: fail and sends the change back to I1-I3 before any V-task may pass`; evidence=`openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt`

## Phase 3: Post-review deterministic verification

- [x] V1 Verify scenario "Reviewing a hot path" with `python3 - <<'PY'
from pathlib import Path
lines = Path('docs/performance-review.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Workload and hot path',
    '## Napkin math',
    '## Resource sketch: network, disk, memory, CPU',
    '## Cost × frequency ranking',
    '## Control plane vs data plane',
    '## Batching, limits, queues, and backpressure',
    '## CPU predictability and compiler luck',
    '## Measurement and validation',
]
assert headings == expected, f'unexpected guide headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/performance-guide-hot-path.txt`
- [x] V3 Verify scenario "Estimating storage or queue cost" with `python3 - <<'PY'
from pathlib import Path
lines = Path('templates/performance-sketch.md').read_text().splitlines()
headings = [line for line in lines if line.startswith('## ')]
expected = [
    '## Workload or entry point',
    '## Assumptions and units',
    '## Bandwidth and latency sketch',
    '## Resource budgets or limits',
    '## Cost × frequency ranking',
    '## Control-plane vs data-plane split',
    '## Batching, limits, queues, and backpressure',
    '## Measurement or validation plan',
    '## Open risks and non-goals',
]
assert headings == expected, f'unexpected template headings: {headings}'
print('\n'.join(headings))
PY`; save output at `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/performance-template-headings.txt`
- [x] V4 Verify scenario "Reader finds the performance guide from README" with `test -f docs/performance-review.md && test -f templates/performance-sketch.md && rg -n '^\[Performance review guide\]\(docs/performance-review\.md\)$' README.md && rg -n '^\[Performance sketch template\]\(templates/performance-sketch\.md\)$' README.md`; save output at `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/readme-performance-links.txt`
- [x] V5 Verify scenario "Contributor sees performance doc touchpoints" with `test -f docs/performance-review.md && test -f templates/performance-sketch.md && test -f openspec/specs/performance/spec.md && rg -n '^\[Performance review guide\]\(performance-review\.md\)$' docs/contributor-guide.md && rg -n '^\[Performance sketch template\]\(\.\./templates/performance-sketch\.md\)$' docs/contributor-guide.md && rg -n '^\[Performance spec contract\]\(\.\./openspec/specs/performance/spec\.md\)$' docs/contributor-guide.md`; save output at `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/contributor-performance-links.txt`
- [x] V6 Verify scenario "Saved review record keeps required labels" with `test -f openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^reviewer role: Tiger Style maintainer$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^reviewed artifact paths: docs/performance-review.md, templates/performance-sketch.md$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^question 2: pass - guide covers every required performance topic from Public Tiger Style Performance Review Guide under the designated headings$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^question 3: pass - template covers every required field, including resource budgets or limits per relevant resource, and every section stays a fill-in field or prompt rather than free-form prose$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^question 4: pass - overall guide guidance, not only per-section boilerplate, says static lints and benchmarks do not replace workload reasoning$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt && rg -n '^verdict: pass$' openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt`; save output at `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review-labels.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Reviewing a hot path` -> `V1` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/performance-guide-hot-path.txt`
- `Scenario: Auditing queue or buffer growth` -> `H1` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt`
- `Scenario: Estimating storage or queue cost` -> `V3` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/performance-template-headings.txt`
- `Scenario: Reader finds the performance guide from README` -> `V4` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/readme-performance-links.txt`
- `Scenario: Contributor sees performance doc touchpoints` -> `V5` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/contributor-performance-links.txt`
- `Scenario: Reviewer signs off guide quality` -> `H1` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt`
- `Scenario: Saved review record keeps required labels` -> `V6` -> `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review-labels.txt`
