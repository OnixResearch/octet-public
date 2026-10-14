## Context

Octet emits or references SARIF, sidecar findings, lint-cache status, semantic fingerprints, executable transcript receipts, UI snapshots, autofix fixtures, and formal-spec experiments. These are not equivalent. A reviewer needs to know the evidence route for each claim.

The taxonomy is a compatibility-sensitive artifact contract, so Rust should own the emitted DTOs and generated Nickel contracts should validate persisted receipt families. Human-authored policy knobs remain Nickel-authored.

## Approach

1. Add a pure `EvidenceTier` model with explicit variants and string discriminants.
2. Include the tier in sidecar finding records and any new evidence summaries that claim check support.
3. For `formal_proof`, require verifier/proof artifact identity fields so the tier cannot be used as a marketing label.
4. For `manual_review_required`, require a human-action message and no success implication.
5. Register the receipt/config family in `standards/policies/registry.ncl` as Rust-derived if Rust emits it after execution.
6. Add positive serialization fixtures for every tier and negative fixtures for unknown tiers and unsupported proof-tier records.

## Risks

- Adding a required field to existing sidecars can break consumers. Mitigate by adding the field to new artifact schema versions or by defaulting missing historical records to `manual_review_required` only in migration tools.
- Teams may misuse `formal_proof`. Mitigate with schema validation requiring proof artifact identity.
- Too many tiers can reduce clarity. Start with the bounded set and require spec changes for additions.

## Validation

- Positive: every defined tier serializes and validates through Rust DTO tests and generated contract checks.
- Negative: unknown tiers fail validation; `formal_proof` without proof artifact identity fails validation.
- Cairn validation and gates pass for this change.
