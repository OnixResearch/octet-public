## Why

Current-head Valence evidence refreshes can discover stale checked-in object-corpus receipt fixtures only after a broader sweep runs. The existing aggregate gate catches the drift, but the failure is not named as a first-class prevention rail, so future THIR/MIR/object changes can land implementation commits before the fixture-refresh obligation is obvious.

## What Changes

- Add a named Nix check for object-corpus fixture freshness.
- Compose the named freshness check into the aggregate Valence object-evidence gate with an operator-visible PASS line.
- Document that Valence object-evidence changes touching object/CLI/fingerprint-consumer or THIR/MIR inventories must run the freshness rail before closeout.

## Scope

In scope: Nix check wiring, aggregate PASS output, contributor/task-template guidance, and deterministic verification.

Out of scope: changing Function Object v1 semantics, changing receipt schemas, broad CI redesign, or claiming correctness/semantic equivalence.

## Verification

Run the named freshness check, aggregate Valence object-evidence check, OpenSpec strict validation, and whitespace checks.
