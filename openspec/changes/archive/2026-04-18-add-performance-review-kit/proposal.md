# Add Tiger Style Performance Review Kit

## Why

Tiger Style treats performance as a first-class design concern: do napkin math early, reason about bandwidth and latency, separate control plane from data plane, batch work, rank costs by resource and frequency, and keep CPU paths predictable. This repo currently captures that material mostly as skill/spec prose. It does not yet ship a stable public performance guide or a copyable sketch template that turns those principles into reviewable artifacts.

Result: tigerstyle gives consumers strong safety/style enforcement, but weak support for the performance half of Tiger Style. This is now the largest remaining gap in the repo's public Tiger Style surface.

## What Changes

- Add a public performance guide that turns Tiger Style performance rules into a review checklist.
- Add a copyable performance-sketch template for napkin math, resource budgets, batching, and bounds.
- Link the guide and template from the README, and link the guide, template, and the existing canonical main performance spec contract at `openspec/specs/performance/spec.md` from the contributor docs so readers can find them without digging through archived changes or private skill files.
- Keep this change doc-first. It does not add compile-time performance lints, benchmark harnesses, or CI perf gates.

## Capabilities

### New Capabilities

- `performance-review-kit`: Maintainers and consumers can review hot paths with a stable Tiger Style performance checklist.
- `performance-sketch-template`: Maintainers can capture napkin math, resource budgets, control-plane/data-plane splits, batching, and validation plans in one reusable document.

## Impact

- **Files**: `README.md`, `docs/performance-review.md`, `docs/contributor-guide.md`, `templates/performance-sketch.md`, `openspec/changes/archive/2026-04-18-add-performance-review-kit/specs/performance/spec.md`, `openspec/changes/archive/2026-04-18-add-performance-review-kit/specs/documentation/spec.md`
- **APIs**: None.
- **Existing contracts**: `openspec/specs/performance/spec.md` already exists as the canonical main performance spec; contributor docs will link to that existing file rather than the change-local delta.
- **Dependencies**: None.
- **Testing**: Deterministic checks on exact section headings and literal Markdown link destinations, plus a saved human/oracle review record at `openspec/changes/archive/2026-04-18-add-performance-review-kit/evidence/human-performance-review.txt` with explicit pass/fail questions.
