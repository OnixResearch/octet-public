# Add Tiger Style Resilience Review Kit

## Why

Tiger Style already pushes explicit failure handling, bounded resources, and
reviewable concurrency or shell boundaries. But the repo does not yet ship one
stable public resilience guide that helps maintainers climb from a recurring
runtime symptom to the design decision underneath it. It also lacks a copyable
artifact for recording a five-whys chain, blast radius, fail-open versus
fail-closed choice, and retry or replay assumptions.

Result: tigerstyle can flag some fragile code patterns, but it gives weaker
support for incident review, design-time resilience reasoning, and explicit
failure semantics.

## What Changes

- Add a public resilience review guide at `docs/resilience-review.md`.
- Add a copyable five-whys architecture template at
  `templates/five-whys-architecture.md`.
- Link the guide and template from `README.md` and
  `docs/contributor-guide.md`.
- Keep this change doc-first. It does not add runtime retry frameworks,
  incident tooling, or chaos harnesses.

## Capabilities

### New Capabilities

- `resilience-review-kit`: Maintainers and consumers can review incidents and
  failure paths with a stable Tiger Style checklist.
- `five-whys-architecture-template`: Maintainers can capture symptom,
  workaround, design cause, blast radius, and follow-up decisions in one
  reusable document.

## Impact

- **Files**: `README.md`, `docs/resilience-review.md`,
  `docs/contributor-guide.md`, `templates/five-whys-architecture.md`,
  `openspec/changes/archive/2026-04-18-add-resilience-review-kit/specs/resilience/spec.md`,
  `openspec/changes/archive/2026-04-18-add-resilience-review-kit/specs/documentation/spec.md`,
  `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`
- **APIs**: None.
- **Existing contracts**: No canonical main resilience spec exists yet in
  `openspec/specs/`; this change adds the resilience contract through a
  change-local delta, while contributor docs in this change link only to the
  new public guide and template.
- **Dependencies**: None.
- **Testing**: Deterministic checks on exact section headings, literal
  Markdown link destinations, and exact human-review label lines, plus a saved
  human/oracle review record at
  `openspec/changes/archive/2026-04-18-add-resilience-review-kit/evidence/human-resilience-review.txt`
  whose explicit pass/fail questions cover per-heading reviewer prompts,
  symptom-to-design reasoning, follow-up ownership, and fill-in-only template
  sections.
