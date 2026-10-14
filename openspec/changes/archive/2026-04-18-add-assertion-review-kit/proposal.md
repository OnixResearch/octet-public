# Add Tiger Style Assertion Review Kit

## Why

Tiger Style has strong opinions about assertions: assert arguments, return
values, and invariants; pair assertions across boundaries; cover both positive
and negative space; split compound checks; use compile-time assertions when the
design can be checked before runtime; and test invalid paths explicitly. This
repo currently exposes only a narrow slice of that philosophy through shipped
lints and skill/spec prose. It does not yet ship a stable public assertion
review guide or a copyable checklist that turns those rules into reviewable
artifacts.

## What Changes

- This active change covers only the assertion philosophy / pair-assertion
  sibling requested by the user; the bounded-allocation sibling is
  `openspec/changes/add-bounds-and-pacing-review-kit/` and the concurrency
  sibling is `openspec/changes/add-concurrency-state-review-kit/`.
- Add a public assertion review guide that turns Tiger Style assertion rules
  into a review checklist.
- Add a copyable assertion checklist template for function contracts,
  paired checks, positive/negative space, compile-time assertions, and
  invalid-path testing.
- Link the guide and template from the README, and link the guide, template,
  and the existing canonical main safety spec contract at
  `openspec/specs/safety/spec.md` from the contributor docs.
- Keep this change doc-first. It does not add new assertion lints,
  macro rewrites, or CI assertion gates.

## Capabilities

### New Capabilities

- `assertion-review-kit`: Maintainers and consumers can review code with a
  stable Tiger Style assertion checklist.
- `assertion-checklist-template`: Maintainers can capture contract checks,
  paired assertion sites, and invalid-path coverage in one reusable document.

## Impact

- **Files**: `README.md`, `docs/assertion-review.md`,
  `docs/contributor-guide.md`, `templates/assertion-checklist.md`,
  `openspec/changes/archive/2026-04-18-add-assertion-review-kit/specs/safety/spec.md`,
  `openspec/changes/archive/2026-04-18-add-assertion-review-kit/specs/documentation/spec.md`
- **APIs**: None.
- **Existing contracts**: `openspec/specs/safety/spec.md` already exists as
  the canonical main safety spec; contributor docs will link to that existing
  file rather than the change-local delta.
- **Dependencies**: None.
- **Testing**: Deterministic checks saved under `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/` for exact section headings, literal Markdown link destinations, and exact labeled-line acceptance at `human-assertion-review-labels.txt`, plus a saved human/oracle review record at `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt` with explicit pass/fail questions.
