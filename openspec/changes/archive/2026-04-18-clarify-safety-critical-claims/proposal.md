# Clarify Tiger Style Safety-Critical Claims

## Why

Tiger Style already ships safety lints, verification-flavored guidance, and
review rules that matter for serious systems. That surface is useful, but it is
also easy to over-read. A clean tigerstyle run is not a certification, not a
formal proof, and not a complete safety case. This repo does not yet ship one
stable public page that says exactly what Tiger Style can claim, what it cannot
claim, and how maintainers should write safety-sensitive docs or review notes
without implying unsupported guarantees.

## What Changes

- Add a public safety-critical claims guide at `docs/safety-critical.md`.
- Link that guide from `README.md` and `docs/contributor-guide.md`.
- Link the existing canonical main safety spec contract at
  `openspec/specs/safety/spec.md` from `docs/contributor-guide.md`.
- Save deterministic evidence artifacts under
  `openspec/changes/clarify-safety-critical-claims/evidence/` for headings,
  public links, and review-label validation.
- Keep this change doc-first. It does not add new safety lints, a
  certification playbook, or legal/regulatory workflow.

## Capabilities

### New Capabilities

- `safety-critical-claims-guide`: Maintainers and consumers can point to one
  stable public guide that explains the supported Tiger Style claim surface.

## Impact

- **Files**: `README.md`, `docs/safety-critical.md`,
  `docs/contributor-guide.md`,
  `openspec/changes/clarify-safety-critical-claims/specs/safety/spec.md`,
  `openspec/changes/clarify-safety-critical-claims/specs/documentation/spec.md`,
  `openspec/changes/clarify-safety-critical-claims/evidence/safety-critical-guide-headings.txt`,
  `openspec/changes/clarify-safety-critical-claims/evidence/readme-safety-critical-links.txt`,
  `openspec/changes/clarify-safety-critical-claims/evidence/contributor-safety-critical-links.txt`,
  `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`,
  `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review-labels.txt`
- **APIs**: None.
- **Existing contracts**: `openspec/specs/safety/spec.md` already exists as
  the canonical main safety spec; contributor docs will link to that existing
  file rather than the change-local delta.
- **Dependencies**: None.
- **Testing**: Deterministic checks on exact section headings, literal
  Markdown link destinations, exact human-review label lines, and saved
  deterministic evidence artifacts, plus a saved human/oracle review record at
  `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`
  whose explicit pass/fail questions cover named-evidence guidance,
  per-heading maintainer prompts or wording rules, and responsible claim
  boundaries.
