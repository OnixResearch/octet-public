## Why

Salt's most useful product lesson is contract-first framing: a reader sees a precondition, a postcondition, and a clear story for how the tool proves or checks it. Octet already has the raw ingredients for a Rust-native version of that story: lints, standards policy, proof cards, Verus proof rails, provenance, and artifact verification. Those ingredients should be presented as a progressive assurance ladder instead of scattered features.

Octet should define a "checked contract" vocabulary for Rust systems code without inventing a new language syntax or overclaiming SMT-backed certainty. A checked contract should be a scoped requirement, invariant, or review obligation that maps to an evidence tier and a command reviewers can run.

## What Changes

- Define Octet's progressive assurance ladder from advisory lint findings through release-gated artifacts and scoped formal-proof receipts.
- Define the fields of a checked-contract record: requirement or policy ID, source scope, invariant/failure shape, evidence tier, command, artifact or receipt path, and manual-review boundary.
- Explain how contract-like claims flow through existing Octet surfaces such as lint diagnostics, standards policy, Verus proof cards, provenance, and artifact verification.
- Specify diagnostic UX expectations for high-confidence findings: diagnostics should name the failing shape, relevant boundary, and remediation path when the analysis can do so.
- Add positive and negative checks that the ladder preserves proof-vs-validation boundaries and does not imply language-level contracts, solver completeness, or runtime fallback semantics.

## Non-Goals

- Do not add a Salt-style `requires` / `ensures` Rust syntax.
- Do not introduce a general Z3/SMT verifier or make Verus mandatory for starter `cargo octet check` usage.
- Do not emit runtime checks as a fallback for unproved contracts.
- Do not change existing receipt schemas, lint defaults, or proof execution semantics in this change unless a follow-up implementation Cairn explicitly scopes that behavior.

## Impact

- **Files**: `README.md`, `docs/trust-boundary-evidence.md`, `docs/lint-reference.md`, `docs/consumer-guide.md`, `verus/README.md`, proof-card docs or fixtures, and deterministic doc-check helpers.
- **Behavior**: docs/specification first; any runner or diagnostic changes should be split into follow-up implementation changes after the ladder is accepted.
- **Testing**: positive documentation checks for ladder stages and checked-contract fields; negative wording checks for language-level contract, solver-completeness, runtime-fallback, and whole-program proof claims; Cairn validation and gates.
