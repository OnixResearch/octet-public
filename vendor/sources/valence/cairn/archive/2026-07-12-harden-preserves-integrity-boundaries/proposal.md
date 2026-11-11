## Why

Valence currently derives Preserves evidence hash material by filtering rendered lines that contain self-hash field names. Payload content can therefore disappear from the digest merely by containing `receipt_hash` or `preserves_content_hash`, allowing distinct envelopes to share one accepted identity. The same trust boundary relies on prose matching to reject overclaims, so contradictory text can satisfy required non-claim phrases while still asserting a stronger claim.

These are integrity failures in the canonical Evidence IR boundary. They must be repaired before downstream evidence-chain, Mantle binding, or Cairn readiness work treats affected receipts as trustworthy.

## What Changes

- Replace rendered-line filtering with parsed Preserves field removal and canonical packed encoding.
- Version the corrected hash profile and reject legacy identities unless an explicit migration validates the original envelope bytes.
- Replace prose-based claim authority with typed claim classes and typed prohibited-capability checks.
- Add collision, contradictory-claim, malformed-envelope, migration, and canonicalization fixtures.
- Add a checked-in CI workflow whose verification scope is `nix flake check`.

## Impact

- **Files**: Preserves evidence canonicalization, Evidence IR claim-boundary types, migration adapters, fixtures, docs, Nix checks, and CI workflow.
- **Compatibility**: receipts produced by the vulnerable profile are not silently grandfathered; migration is explicit and byte-bound.
- **Ordering**: this change is an upstream prerequisite for new evidence-chain profiles that consume Preserves identities.
- **Claims**: passing validation proves canonical envelope identity and typed claim-policy conformance only, not evidence truth, behavioral correctness, verifier soundness, or release eligibility.
