# Add Tiger Style System Comprehension Guidance

## Why

Tiger Style already cares about boundaries, explicit ownership, contributor
traceability, and reviewable artifacts. But it does not yet ship one stable
public guide that teaches maintainers how to explain a system in the order that
new readers can actually absorb it: structure first, then function, then
selected depth. The repo also lacks a copyable walkthrough template for major
components, dependency edges, end-to-end flow, and weak spots.

Result: tigerstyle can help enforce code policy, but it gives weaker support
for architecture legibility, onboarding, and system explanation.

## What Changes

- Add a public system comprehension guide at `docs/system-comprehension.md`.
- Add a copyable walkthrough template at `templates/system-walkthrough.md`.
- Link the guide and template from `README.md` and `docs/contributor-guide.md`.
- Keep this change doc-first. It does not add automatic architecture scoring,
  topology lints, or generated diagrams.

## Capabilities

### New Capabilities

- `system-comprehension-guide`: Maintainers and consumers can review whether a
  codebase teaches structure before function before depth.
- `system-walkthrough-template`: Maintainers can capture system slice,
  boundaries, flow, weak spots, and open questions in one reusable document.

## Impact

- **Files**: `README.md`, `docs/system-comprehension.md`,
  `docs/contributor-guide.md`, `templates/system-walkthrough.md`,
  `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/specs/architecture/spec.md`,
  `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/specs/documentation/spec.md`,
  `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`
- **APIs**: None.
- **Existing contracts**: No canonical main architecture spec exists yet in
  `openspec/specs/`; this change adds the architecture contract through a
  change-local delta, while contributor docs in this change link only to the
  new public guide and template.
- **Dependencies**: None.
- **Testing**: Deterministic checks on exact section headings, literal
  Markdown link destinations, and exact human-review label lines, plus a saved
  human/oracle review record at
  `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`
  whose explicit pass/fail questions cover all required guide sections,
  per-heading reviewer prompts, and fill-in-only template sections.
