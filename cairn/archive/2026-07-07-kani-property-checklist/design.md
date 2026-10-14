# Design: Kani property checklist

## Approach

The checklist is a review and evidence-shaping artifact. It does not require Kani for every project, but when Kani or property tests are used, it gives reviewers stable categories and expected positive/negative coverage.

## Property classes

The initial checklist includes:

- normalization and idempotence;
- algebraic laws and identity elements;
- encode/decode and split/join round trips;
- validity-range and calendar/format predicates;
- loop termination and bounded progress;
- `Eq`/`Ord`/`Hash` consistency;
- overflow, panic, and boundary behavior.

## Receipt integration

Kani receipts may include a property-class label from the checklist. Unknown labels remain review metadata unless the policy is extended.

## Positive and negative coverage

Each adopted checklist item names the expected successful behavior and at least one failure mode: malformed input, invalid range, trait-law violation, overflow boundary, or stale receipt.

## What is explicitly not provided

- No claim that every checklist class applies to every module.
- No automatic proof generation.
- No replacement for domain-specific requirements or proof rails.
