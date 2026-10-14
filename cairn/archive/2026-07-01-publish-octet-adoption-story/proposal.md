## Why

Salt's public story is compelling because it gives readers a crisp promise, concrete systems built with the tool, visible benchmark methodology, and a short path from first command to proof-shaped evidence. Octet already has stronger adoption leverage for Rust teams because it works with existing workspaces, runner metadata, lint gates, standards checks, Verus receipts, and artifact verification. The public docs do not yet present that advantage as one clear adoption story.

Octet should adapt Salt's packaging lessons without copying its new-language or unqualified mathematical-certainty claims. The product story should say what Octet checks, what evidence reviewers inspect, which downstream systems have exercised the path, and which claims remain out of scope.

## What Changes

- Add or revise a primary adoption-story surface for Octet with a sharper Rust-compatible promise: evidence-backed Rust systems code checked before release.
- Add a "Built with Octet" case-study format that names checked scope, commands, findings or suppressions, evidence artifacts, rollout status, and explicit scope boundaries.
- Seed the case-study surface from existing archived downstream evidence where available, especially the Aspen phase-1 rollout archive path called out in repo guidance.
- Add a methodology section for performance and adoption evidence: measured command/runtime overhead, cache behavior, CI impact, raw commands, and evidence artifact paths.
- Add deterministic documentation checks that required positioning and case-study fields are present and forbidden overclaims are absent.

## Non-Goals

- Do not present Octet as a new programming language or Rust replacement.
- Do not claim mathematical certainty, whole-program correctness, a verified runtime, or certification from lint/policy evidence.
- Do not add new lint behavior, proof execution behavior, receipt schemas, runner commands, or package-manager behavior.
- Do not depend on live external competitor pages for validation; external comparisons are inspiration, not a build-time input.

## Impact

- **Files**: `README.md`, `docs/consumer-guide.md`, `docs/trust-boundary-evidence.md`, `docs/downstream-rollout.md`, a possible new docs page such as `docs/adoption-story.md`, and deterministic doc-check fixtures or scripts.
- **Behavior**: documentation and evidence presentation only; runner and standards behavior remain unchanged.
- **Testing**: positive documentation inventory for hero promise, case-study fields, methodology fields, and Aspen archive citation; negative wording check for unsupported correctness/proof/language-replacement claims; Cairn validation and gates.
